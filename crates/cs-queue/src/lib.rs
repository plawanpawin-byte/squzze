//! Persistent job queue: scheduler, worker threads, cancel/retry and the FFmpeg-backed encoder.

use cs_ffmpeg::{build_args_full, build_args_pass, plan_output, sha256_file, strip_metadata, run_encode_logged, temp_path, verify_output_with, AudioChoice, AudioMode, HwKind, Overwrite, PlanError, Preset, Progress, RunError, Trim, Verify};
use cs_probe::{probe, validate_input};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub source: String,
    pub file_name: String,
    pub duration_secs: Option<f64>,
    pub preset: Preset,
    pub dest_dir: Option<String>,
    pub template: String,
    pub overwrite: Overwrite,
    pub state: JobState,
    pub progress: f32,
    pub fps: Option<f64>,
    pub speed: Option<f64>,
    pub eta_secs: Option<f64>,
    pub error: Option<String>,
    pub output: Option<String>,
    pub note: Option<String>,
    pub attempt: u32,
    pub created_at: u64,
    /// Temp file being written (kept so a crash can be cleaned up at next start).
    pub temp: Option<String>,
    /// In/out points applied to the source, seconds. `duration_secs` is the resulting length.
    #[serde(default)]
    pub start_secs: Option<f64>,
    #[serde(default)]
    pub end_secs: Option<f64>,
    #[serde(default)]
    pub source_size: Option<u64>,
    #[serde(default)]
    pub started_ms: Option<u64>,
    #[serde(default)]
    pub finished_ms: Option<u64>,
    /// The exact ffmpeg command line used (for support and reproducibility).
    #[serde(default)]
    pub cmdline: Option<String>,
    /// Last lines FFmpeg printed.
    #[serde(default)]
    pub log: Vec<String>,
    /// What ffprobe found in the finished output compared with the preset.
    #[serde(default)]
    pub verify: Option<Verify>,
    /// Which audio stream, copy/convert/remove, downmix.
    #[serde(default)]
    pub audio: AudioChoice,
    /// Do not copy tags/chapters from the source.
    #[serde(default)]
    pub strip_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJob {
    pub source: String,
    pub duration_secs: Option<f64>,
    pub preset: Preset,
    pub dest_dir: Option<String>,
    pub template: Option<String>,
    pub overwrite: Option<Overwrite>,
    #[serde(default)]
    pub start_secs: Option<f64>,
    #[serde(default)]
    pub end_secs: Option<f64>,
    #[serde(default)]
    pub audio: AudioChoice,
    #[serde(default)]
    pub strip_metadata: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub max_parallel: u32,
    pub use_hw: bool,
    pub default_dest: Option<String>,
    pub template: String,
    pub overwrite: Overwrite,
    /// Preset ids the user starred (built-in or custom).
    #[serde(default)]
    pub favorites: Vec<String>,
    /// Remove source tags (title, location, camera info) and chapters from every output.
    #[serde(default)]
    pub strip_metadata: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { max_parallel: 1, use_hw: true, default_dest: None, template: "{name}-{preset}".into(), overwrite: Overwrite::Rename, favorites: vec![], strip_metadata: false }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Settings {
        self.max_parallel = self.max_parallel.clamp(1, 4);
        if self.template.trim().is_empty() || self.template.chars().count() > 120 {
            self.template = "{name}-{preset}".into();
        }
        if let Some(d) = &self.default_dest {
            if !Path::new(d).is_absolute() {
                self.default_dest = None;
            }
        }
        // only plausible preset ids, no duplicates, bounded
        let mut seen = std::collections::HashSet::new();
        self.favorites.retain(|id| !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') && seen.insert(id.clone()));
        self.favorites.truncate(200);
        self
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Updated(Box<Job>),
    Removed(u64),
    /// Queue ran out of work.
    Idle,
}

pub enum Report {
    Cmd(String),
    Log(Vec<String>),
    Verify(Verify),
    Temp(String),
    Progress(Progress),
    Note(String),
}

pub trait Encoder: Send + Sync + 'static {
    /// Run one job. Returns the final output path.
    fn encode(&self, job: &Job, cancel: &AtomicBool, report: &mut dyn FnMut(Report)) -> Result<String, RunError>;
}

#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    next_id: u64,
    jobs: Vec<Job>,
}

struct Inner {
    jobs: Vec<Job>,
    next_id: u64,
    flags: HashMap<u64, Arc<AtomicBool>>,
    active: bool,
    shutdown: bool,
    max_parallel: usize,
}

struct Shared {
    inner: Mutex<Inner>,
    cv: Condvar,
    encoder: Arc<dyn Encoder>,
    on_event: Box<dyn Fn(Event) + Send + Sync>,
    store: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Queue {
    shared: Arc<Shared>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Validate in/out points and return the resulting length (when it can be known).
pub fn effective_duration(source_secs: Option<f64>, start: Option<f64>, end: Option<f64>) -> Result<Option<f64>, String> {
    for v in [start, end].into_iter().flatten() {
        if !v.is_finite() || v < 0.0 {
            return Err("in/out points must be positive numbers".into());
        }
    }
    let s0 = start.unwrap_or(0.0);
    if let Some(e) = end {
        if e <= s0 {
            return Err("the out point must be after the in point".into());
        }
        if let Some(d) = source_secs {
            if e > d + 0.5 {
                return Err("the out point is beyond the end of the file".into());
            }
        }
    }
    if let (Some(d), None) = (source_secs, end) {
        if s0 >= d {
            return Err("the in point is beyond the end of the file".into());
        }
    }
    Ok(match (source_secs, end) {
        (_, Some(e)) => Some(e - s0),
        (Some(d), None) => Some(d - s0),
        _ => None,
    })
}

pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)
}

impl Queue {
    /// `store` is the JSON file used for persistence (None = in-memory only).
    pub fn new(store: Option<PathBuf>, max_parallel: u32, encoder: Arc<dyn Encoder>, on_event: impl Fn(Event) + Send + Sync + 'static) -> Queue {
        let mut jobs = vec![];
        let mut next_id = 1;
        if let Some(p) = &store {
            if let Ok(text) = std::fs::read_to_string(p) {
                match serde_json::from_str::<Saved>(&text) {
                    Ok(s) if s.version == 1 => {
                        jobs = s.jobs;
                        next_id = s.next_id.max(jobs.iter().map(|j| j.id + 1).max().unwrap_or(1));
                    }
                    _ => {
                        let _ = std::fs::rename(p, p.with_extension("corrupt.json"));
                    }
                }
            }
        }
        for j in jobs.iter_mut() {
            if let Some(t) = j.temp.take() {
                let _ = std::fs::remove_file(t);
            }
            if j.state == JobState::Running {
                j.state = JobState::Queued;
                j.progress = 0.0;
                j.note = Some("Re-queued after restart".into());
            }
        }
        let shared = Arc::new(Shared {
            inner: Mutex::new(Inner { jobs, next_id, flags: HashMap::new(), active: false, shutdown: false, max_parallel: max_parallel.clamp(1, 4) as usize }),
            cv: Condvar::new(),
            encoder,
            on_event: Box::new(on_event),
            store,
        });
        let q = Queue { shared: shared.clone() };
        let s2 = shared;
        std::thread::spawn(move || dispatcher(s2));
        q
    }

    pub fn jobs(&self) -> Vec<Job> {
        self.shared.inner.lock().unwrap().jobs.clone()
    }

    pub fn set_max_parallel(&self, n: u32) {
        let mut g = self.shared.inner.lock().unwrap();
        g.max_parallel = n.clamp(1, 4) as usize;
        self.shared.cv.notify_all();
    }

    pub fn add(&self, new: NewJob, settings: &Settings) -> Result<u64, String> {
        new.preset.validate().map_err(|e| e.to_string())?;
        let src = validate_input(Path::new(&new.source)).map_err(|e| e.to_string())?;
        let template = new.template.unwrap_or_else(|| settings.template.clone());
        let duration = effective_duration(new.duration_secs, new.start_secs, new.end_secs)?;
        let source_size = std::fs::metadata(&src).ok().map(|m| m.len());
        let mut g = self.shared.inner.lock().unwrap();
        let id = g.next_id;
        g.next_id += 1;
        let job = Job {
            id,
            file_name: src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            source: src.to_string_lossy().into_owned(),
            duration_secs: duration,
            preset: new.preset,
            dest_dir: new.dest_dir.or_else(|| settings.default_dest.clone()),
            template,
            overwrite: new.overwrite.unwrap_or(settings.overwrite),
            state: JobState::Queued,
            progress: 0.0,
            fps: None,
            speed: None,
            eta_secs: None,
            error: None,
            output: None,
            note: None,
            attempt: 0,
            created_at: now(),
            temp: None,
            start_secs: new.start_secs,
            end_secs: new.end_secs,
            source_size,
            started_ms: None,
            finished_ms: None,
            cmdline: None,
            log: vec![],
            verify: None,
            audio: new.audio,
            strip_metadata: new.strip_metadata.unwrap_or(settings.strip_metadata),
        };
        g.jobs.push(job.clone());
        self.persist(&g);
        drop(g);
        (self.shared.on_event)(Event::Updated(Box::new(job)));
        Ok(id)
    }

    pub fn start(&self) {
        let mut g = self.shared.inner.lock().unwrap();
        g.active = true;
        self.shared.cv.notify_all();
    }

    /// Stop launching new jobs; running ones continue.
    pub fn pause(&self) {
        self.shared.inner.lock().unwrap().active = false;
    }

    pub fn cancel(&self, id: u64) {
        let mut g = self.shared.inner.lock().unwrap();
        if let Some(flag) = g.flags.get(&id) {
            flag.store(true, Ordering::SeqCst);
            return;
        }
        let mut changed = None;
        if let Some(j) = g.jobs.iter_mut().find(|j| j.id == id) {
            if j.state == JobState::Queued {
                j.state = JobState::Cancelled;
                changed = Some(j.clone());
            }
        }
        if changed.is_some() {
            self.persist(&g);
        }
        drop(g);
        if let Some(j) = changed {
            (self.shared.on_event)(Event::Updated(Box::new(j)));
        }
    }

    pub fn cancel_all(&self) {
        let ids: Vec<u64> = self.shared.inner.lock().unwrap().jobs.iter().filter(|j| matches!(j.state, JobState::Queued | JobState::Running)).map(|j| j.id).collect();
        for id in ids {
            self.cancel(id);
        }
    }

    pub fn retry(&self, id: u64) {
        let mut g = self.shared.inner.lock().unwrap();
        let mut changed = None;
        if let Some(j) = g.jobs.iter_mut().find(|j| j.id == id) {
            if matches!(j.state, JobState::Failed | JobState::Cancelled) {
                j.state = JobState::Queued;
                j.progress = 0.0;
                j.error = None;
                j.note = None;
                j.fps = None;
                j.speed = None;
                j.eta_secs = None;
                j.verify = None;
                j.log.clear();
                j.started_ms = None;
                j.finished_ms = None;
                changed = Some(j.clone());
            }
        }
        if changed.is_some() {
            self.persist(&g);
            self.shared.cv.notify_all();
        }
        drop(g);
        if let Some(j) = changed {
            (self.shared.on_event)(Event::Updated(Box::new(j)));
        }
    }

    /// Remove a job that is not running. Returns false if it is running or unknown.
    pub fn remove(&self, id: u64) -> bool {
        let mut g = self.shared.inner.lock().unwrap();
        let Some(pos) = g.jobs.iter().position(|j| j.id == id) else { return false };
        if g.jobs[pos].state == JobState::Running {
            return false;
        }
        g.jobs.remove(pos);
        self.persist(&g);
        drop(g);
        (self.shared.on_event)(Event::Removed(id));
        true
    }

    pub fn clear_finished(&self) {
        let ids: Vec<u64> = self.shared.inner.lock().unwrap().jobs.iter().filter(|j| matches!(j.state, JobState::Done | JobState::Cancelled)).map(|j| j.id).collect();
        for id in ids {
            self.remove(id);
        }
    }

    /// Cancel running jobs for app exit; they stay queued for the next start. Blocks briefly.
    pub fn shutdown(&self) {
        {
            let mut g = self.shared.inner.lock().unwrap();
            g.shutdown = true;
            g.active = false;
            for f in g.flags.values() {
                f.store(true, Ordering::SeqCst);
            }
            self.shared.cv.notify_all();
        }
        for _ in 0..50 {
            if self.shared.inner.lock().unwrap().flags.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let g = self.shared.inner.lock().unwrap();
        self.persist(&g);
    }

    fn persist(&self, g: &Inner) {
        persist(&self.shared, g);
    }
}

fn persist(shared: &Shared, g: &Inner) {
    if let Some(p) = &shared.store {
        let saved = Saved { version: 1, next_id: g.next_id, jobs: g.jobs.clone() };
        if let Ok(bytes) = serde_json::to_vec_pretty(&saved) {
            let _ = write_atomic(p, &bytes);
        }
    }
}

/// Delivery manifest for finished jobs: what was made from what, with sizes, checks and SHA-256.
pub fn manifest_json(jobs: &[Job], app_version: &str) -> serde_json::Value {
    let items: Vec<serde_json::Value> = jobs
        .iter()
        .filter(|j| j.state == JobState::Done)
        .map(|j| {
            let v = j.verify.as_ref();
            serde_json::json!({
                "source": j.source,
                "source_size_bytes": j.source_size,
                "range_secs": [j.start_secs, j.end_secs],
                "preset": { "id": j.preset.id, "name": j.preset.name, "container": j.preset.container },
                "output": j.output,
                "output_size_bytes": v.and_then(|v| v.size_bytes),
                "duration_secs": v.and_then(|v| v.duration_secs),
                "video_codec": v.and_then(|v| v.video_codec.clone()),
                "audio_codec": v.and_then(|v| v.audio_codec.clone()),
                "verified": v.map(|v| v.ok),
                "verify_notes": v.map(|v| v.notes.clone()).unwrap_or_default(),
                "sha256": v.and_then(|v| v.sha256.clone()),
                "encoder_note": j.note,
                "started_ms": j.started_ms,
                "finished_ms": j.finished_ms,
            })
        })
        .collect();
    serde_json::json!({ "manifest_version": 1, "generator": format!("Squeeze {app_version}"), "created": now(), "items": items })
}

fn dispatcher(shared: Arc<Shared>) {
    let mut g = shared.inner.lock().unwrap();
    let mut was_busy = false;
    loop {
        if g.shutdown {
            return;
        }
        let running = g.flags.len();
        if g.active {
            let mut launched = 0;
            while g.flags.len() < g.max_parallel {
                let Some(idx) = g.jobs.iter().position(|j| j.state == JobState::Queued) else { break };
                let flag = Arc::new(AtomicBool::new(false));
                let id = g.jobs[idx].id;
                g.jobs[idx].state = JobState::Running;
                g.jobs[idx].attempt += 1;
                g.jobs[idx].progress = 0.0;
                g.jobs[idx].error = None;
                g.jobs[idx].note = None;
                g.jobs[idx].verify = None;
                g.jobs[idx].log.clear();
                g.jobs[idx].started_ms = Some(now_ms());
                g.jobs[idx].finished_ms = None;
                g.flags.insert(id, flag.clone());
                let job = g.jobs[idx].clone();
                persist(&shared, &g);
                (shared.on_event)(Event::Updated(Box::new(job.clone())));
                let s2 = shared.clone();
                std::thread::spawn(move || worker(s2, job, flag));
                launched += 1;
            }
            let busy = launched > 0 || running > 0 || !g.flags.is_empty();
            let has_queued = g.jobs.iter().any(|j| j.state == JobState::Queued);
            if was_busy && !busy && !has_queued {
                g.active = false;
                (shared.on_event)(Event::Idle);
            }
            was_busy = busy || has_queued;
        } else {
            was_busy = false;
        }
        g = shared.cv.wait_timeout(g, Duration::from_millis(500)).unwrap().0;
    }
}

fn update(shared: &Shared, id: u64, f: impl FnOnce(&mut Job), save: bool) {
    let mut g = shared.inner.lock().unwrap();
    let mut snapshot = None;
    if let Some(j) = g.jobs.iter_mut().find(|j| j.id == id) {
        f(j);
        snapshot = Some(j.clone());
    }
    if save {
        persist(shared, &g);
    }
    drop(g);
    if let Some(j) = snapshot {
        (shared.on_event)(Event::Updated(Box::new(j)));
    }
}

fn worker(shared: Arc<Shared>, job: Job, flag: Arc<AtomicBool>) {
    let id = job.id;
    let sh = shared.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        shared.encoder.encode(&job, &flag, &mut |r| match r {
            Report::Temp(t) => update(&sh, id, |j| j.temp = Some(t), true),
            Report::Cmd(c) => update(&sh, id, |j| j.cmdline = Some(c), false),
            Report::Log(l) => update(&sh, id, |j| j.log = l, false),
            Report::Verify(v) => update(&sh, id, |j| j.verify = Some(v), false),
            Report::Note(n) => update(&sh, id, |j| j.note = Some(n), false),
            Report::Progress(p) => update(
                &sh,
                id,
                |j| {
                    j.progress = p.percent;
                    j.fps = p.fps;
                    j.speed = p.speed;
                    j.eta_secs = p.eta_secs;
                },
                false,
            ),
        })
    }));
    let shutting_down = shared.inner.lock().unwrap().shutdown;
    let result = result.unwrap_or_else(|_| Err(RunError::Failed("internal error while encoding".into())));
    update(
        &shared,
        id,
        |j| {
            j.temp = None;
            j.fps = None;
            j.speed = None;
            j.eta_secs = None;
            j.finished_ms = Some(now_ms());
            match result {
                Ok(out) => {
                    j.state = JobState::Done;
                    j.progress = 100.0;
                    j.output = Some(out);
                }
                Err(RunError::Cancelled) if shutting_down => {
                    j.state = JobState::Queued;
                    j.progress = 0.0;
                }
                Err(RunError::Cancelled) => j.state = JobState::Cancelled,
                Err(e) => {
                    j.state = JobState::Failed;
                    j.error = Some(e.to_string());
                }
            }
        },
        true,
    );
    shared.inner.lock().unwrap().flags.remove(&id);
    shared.cv.notify_all();
}

/// Real encoder: runs FFmpeg with temp file + atomic rename, hardware first then software fallback,
/// then re-probes the finished file and checks it against the preset.
pub struct FfmpegEncoder {
    pub ffmpeg: PathBuf,
    pub ffprobe: Option<PathBuf>,
    pub hw: Arc<Mutex<Vec<HwKind>>>,
    pub settings: Arc<Mutex<Settings>>,
}

fn remove_quiet(p: &Path) {
    let _ = std::fs::remove_file(p);
}

fn use_hw_requested(settings: &Arc<Mutex<Settings>>) -> bool {
    settings.lock().unwrap().use_hw
}

/// Remove x264/x265/vp9 pass-log files (`<prefix>-0.log`, `.mbtree`, `.cutree` …).
fn remove_pass_logs(prefix: &Path) {
    let (Some(dir), Some(name)) = (prefix.parent(), prefix.file_name().map(|n| n.to_string_lossy().into_owned())) else { return };
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().starts_with(&name) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

fn quote(a: &str) -> String {
    if a.contains(' ') || a.contains('\'') { format!("\"{a}\"") } else { a.to_string() }
}

impl Encoder for FfmpegEncoder {
    fn encode(&self, job: &Job, cancel: &AtomicBool, report: &mut dyn FnMut(Report)) -> Result<String, RunError> {
        let src = validate_input(Path::new(&job.source)).map_err(|e| RunError::Failed(e.to_string()))?;
        let dest = job.dest_dir.as_deref().map(Path::new);
        if let Some(d) = dest {
            std::fs::create_dir_all(d).map_err(|e| RunError::Failed(format!("cannot create destination folder: {e}")))?;
        }
        let fin = plan_output(&src, dest, &job.template, &job.preset.id, &job.preset.container, job.overwrite).map_err(|e| match e {
            PlanError::Exists(p) => RunError::Failed(format!("output already exists: {}", p.display())),
            PlanError::NoDir(m) => RunError::Failed(m),
        })?;
        let tmp = temp_path(&fin);
        report(Report::Temp(tmp.to_string_lossy().into_owned()));

        let hw_kind = {
            let use_hw = self.settings.lock().unwrap().use_hw;
            let list = self.hw.lock().unwrap();
            match (&job.preset.video, use_hw) {
                // 10-bit, two-pass, profile/level and speed are honoured exactly only by the software encoders
                (Some(v), true) if !v.needs_software() => list.iter().copied().find(|k| k.encoder(v.codec).is_some()),
                _ => None,
            }
        };
        let forced_software = use_hw_requested(&self.settings) && job.preset.video.as_ref().is_some_and(|v| v.needs_software());
        let two_pass = job.preset.video.as_ref().is_some_and(|v| v.two_pass);
        report(Report::Note(match hw_kind {
            Some(k) => format!("Encoding with {}", k.label()),
            None if forced_software => "Encoding in software (CPU) — the advanced settings need it".into(),
            None => "Encoding in software (CPU)".into(),
        }));
        let trim = Trim { start: job.start_secs, end: job.end_secs };
        let cmd_text = |args: &[String]| format!("ffmpeg {}", args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" "));
        let attempt = |hw: Option<HwKind>, report: &mut dyn FnMut(Report)| {
            if !two_pass {
                let mut args = build_args_full(&job.preset, &src, &tmp, hw, trim, &job.audio);
                if job.strip_metadata {
                    strip_metadata(&mut args);
                }
                report(Report::Cmd(cmd_text(&args)));
                return run_encode_logged(&self.ffmpeg, &args, job.duration_secs, cancel, |p| report(Report::Progress(p)));
            }
            // two passes: analysis (0-50 %) then the real encode (50-100 %)
            let logp = tmp.with_extension("pass");
            let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
            let a1 = build_args_pass(&job.preset, &src, Path::new(null), None, trim, &job.audio, Some((1, &logp)));
            let mut a2 = build_args_pass(&job.preset, &src, &tmp, None, trim, &job.audio, Some((2, &logp)));
            if job.strip_metadata {
                strip_metadata(&mut a2);
            }
            report(Report::Cmd(format!("{}
{}", cmd_text(&a1), cmd_text(&a2))));
            let cleanup = || remove_pass_logs(&logp);
            let (r1, log1) = run_encode_logged(&self.ffmpeg, &a1, job.duration_secs, cancel, |p| report(Report::Progress(Progress { percent: p.percent * 0.5, ..p })));
            if r1.is_err() {
                cleanup();
                return (r1, log1);
            }
            let out = run_encode_logged(&self.ffmpeg, &a2, job.duration_secs, cancel, |p| report(Report::Progress(Progress { percent: 50.0 + p.percent * 0.5, ..p })));
            cleanup();
            out
        };
        let (mut res, mut log) = attempt(hw_kind, report);
        if let (Err(RunError::Failed(m)), Some(k)) = (&res, hw_kind) {
            remove_quiet(&tmp);
            report(Report::Note(format!("{} failed ({}); retried in software", k.label(), m.chars().take(120).collect::<String>())));
            (res, log) = attempt(None, report);
        } else if let (Ok(()), Some(k)) = (&res, hw_kind) {
            report(Report::Note(format!("Encoded with {}", k.label())));
        } else if res.is_ok() {
            report(Report::Note(if forced_software { "Encoded in software — the advanced settings need it".into() } else { "Encoded in software".into() }));
        }
        report(Report::Log(log));
        match res {
            Ok(()) => {
                let size = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
                if size == 0 {
                    remove_quiet(&tmp);
                    return Err(RunError::Failed("ffmpeg produced an empty file".into()));
                }
                std::fs::rename(&tmp, &fin).map_err(|e| {
                    remove_quiet(&tmp);
                    RunError::Failed(format!("could not write output: {e}"))
                })?;
                if let Some(probe_tool) = &self.ffprobe {
                    let src_info = probe(probe_tool, &src).ok();
                    let src_stream = src_info.as_ref().and_then(|i| i.audio_streams.get(job.audio.stream_index() as usize));
                    let expect_audio = job.audio.mode != AudioMode::None && src_stream.map(|_| true).unwrap_or(src_info.is_none());
                    let copied_codec = if job.audio.mode == AudioMode::Copy { src_stream.map(|a| a.codec.clone()) } else { None };
                    let mut v = match probe(probe_tool, &fin) {
                        Ok(info) => verify_output_with(&info, &job.preset, job.duration_secs, expect_audio, copied_codec.as_deref()),
                        Err(e) => Verify { ok: false, notes: vec![format!("could not read the finished file: {e}")], ..Verify::default() },
                    };
                    v.sha256 = sha256_file(&fin).ok();
                    report(Report::Verify(v));
                }
                Ok(fin.to_string_lossy().into_owned())
            }
            Err(e) => {
                remove_quiet(&tmp);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cs_ffmpeg::builtin_presets;
    use std::sync::atomic::AtomicUsize;

    struct Fake {
        running: AtomicUsize,
        peak: AtomicUsize,
        ms: u64,
        fail_names: Vec<&'static str>,
    }
    impl Encoder for Fake {
        fn encode(&self, job: &Job, cancel: &AtomicBool, report: &mut dyn FnMut(Report)) -> Result<String, RunError> {
            let r = self.running.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(r, Ordering::SeqCst);
            let steps = self.ms / 20;
            let mut out = Ok(format!("{}.out", job.source));
            for i in 0..steps {
                if cancel.load(Ordering::SeqCst) {
                    out = Err(RunError::Cancelled);
                    break;
                }
                report(Report::Progress(Progress { percent: (i as f32 / steps as f32) * 100.0, ..Default::default() }));
                std::thread::sleep(Duration::from_millis(20));
            }
            if self.fail_names.iter().any(|n| job.file_name.contains(n)) {
                out = Err(RunError::Failed("boom".into()));
            }
            self.running.fetch_sub(1, Ordering::SeqCst);
            out
        }
    }

    fn files(n: usize, tag: &str) -> (PathBuf, Vec<PathBuf>) {
        let d = std::env::temp_dir().join(format!("sq-q-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let v = (0..n)
            .map(|i| {
                let p = d.join(format!("f{i}.mp4"));
                std::fs::write(&p, b"x").unwrap();
                p
            })
            .collect();
        (d, v)
    }

    fn newjob(p: &Path) -> NewJob {
        NewJob { source: p.to_string_lossy().into_owned(), duration_secs: Some(1.0), preset: builtin_presets().remove(0), dest_dir: None, template: None, overwrite: None, start_secs: None, end_secs: None, audio: AudioChoice::default(), strip_metadata: None }
    }

    fn wait_for(q: &Queue, f: impl Fn(&[Job]) -> bool) {
        for _ in 0..400 {
            if f(&q.jobs()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("timeout; jobs = {:?}", q.jobs().iter().map(|j| j.state).collect::<Vec<_>>());
    }

    #[test]
    fn respects_parallel_limit_and_finishes_all() {
        let (d, fs) = files(6, "par");
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 120, fail_names: vec![] });
        let q = Queue::new(None, 2, enc.clone(), |_| {});
        for f in &fs {
            q.add(newjob(f), &Settings::default()).unwrap();
        }
        q.start();
        wait_for(&q, |j| j.iter().all(|x| x.state == JobState::Done));
        assert_eq!(enc.peak.load(Ordering::SeqCst), 2);
        q.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn failure_does_not_stop_queue_and_retry_works() {
        let (d, fs) = files(3, "fail");
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 40, fail_names: vec!["f1"] });
        let q = Queue::new(None, 1, enc, |_| {});
        for f in &fs {
            q.add(newjob(f), &Settings::default()).unwrap();
        }
        q.start();
        wait_for(&q, |j| j.iter().all(|x| matches!(x.state, JobState::Done | JobState::Failed)));
        let js = q.jobs();
        assert_eq!(js.iter().filter(|j| j.state == JobState::Failed).count(), 1);
        assert_eq!(js.iter().filter(|j| j.state == JobState::Done).count(), 2);
        assert_eq!(js.iter().find(|j| j.state == JobState::Failed).unwrap().error.as_deref(), Some("boom"));
        let id = js.iter().find(|j| j.state == JobState::Failed).unwrap().id;
        q.retry(id);
        assert_eq!(q.jobs().iter().find(|j| j.id == id).unwrap().state, JobState::Queued);
        q.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn cancel_running_and_queued() {
        let (d, fs) = files(3, "cancel");
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 2000, fail_names: vec![] });
        let q = Queue::new(None, 1, enc, |_| {});
        let ids: Vec<u64> = fs.iter().map(|f| q.add(newjob(f), &Settings::default()).unwrap()).collect();
        q.start();
        wait_for(&q, |j| j[0].state == JobState::Running);
        q.cancel(ids[1]); // queued
        q.cancel(ids[0]); // running
        wait_for(&q, |j| j[0].state == JobState::Cancelled);
        assert_eq!(q.jobs()[1].state, JobState::Cancelled);
        assert!(!q.remove(999));
        q.cancel_all();
        q.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn queue_survives_restart_and_running_becomes_queued() {
        let (d, fs) = files(3, "persist");
        let store = d.join("queue.json");
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 5000, fail_names: vec![] });
        let q = Queue::new(Some(store.clone()), 1, enc.clone(), |_| {});
        for f in &fs {
            q.add(newjob(f), &Settings::default()).unwrap();
        }
        q.start();
        wait_for(&q, |j| j[0].state == JobState::Running);
        q.shutdown(); // simulates app exit while running
        let q2 = Queue::new(Some(store.clone()), 1, enc, |_| {});
        let js = q2.jobs();
        assert_eq!(js.len(), 3);
        assert!(js.iter().all(|j| j.state == JobState::Queued));
        let id = q2.add(newjob(&fs[0]), &Settings::default()).unwrap();
        assert_eq!(id, 4); // ids keep increasing
        q2.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn corrupt_store_is_set_aside_not_fatal() {
        let d = std::env::temp_dir().join(format!("sq-q-corrupt-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let store = d.join("queue.json");
        std::fs::write(&store, b"{ not json").unwrap();
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 20, fail_names: vec![] });
        let q = Queue::new(Some(store.clone()), 1, enc, |_| {});
        assert!(q.jobs().is_empty());
        assert!(d.join("queue.corrupt.json").exists());
        q.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn add_rejects_missing_file_and_bad_preset() {
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 20, fail_names: vec![] });
        let q = Queue::new(None, 1, enc, |_| {});
        assert!(q.add(newjob(Path::new("Z:\\nope.mp4")), &Settings::default()).is_err());
        let (d, fs) = files(1, "badp");
        let mut nj = newjob(&fs[0]);
        nj.preset.container = "exe".into();
        assert!(q.add(nj, &Settings::default()).is_err());
        q.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn trim_validation_and_effective_duration() {
        assert_eq!(effective_duration(Some(10.0), None, None), Ok(Some(10.0)));
        assert_eq!(effective_duration(Some(10.0), Some(2.0), Some(6.0)), Ok(Some(4.0)));
        assert_eq!(effective_duration(Some(10.0), Some(2.0), None), Ok(Some(8.0)));
        assert_eq!(effective_duration(None, Some(1.0), Some(4.0)), Ok(Some(3.0)));
        assert_eq!(effective_duration(None, None, None), Ok(None));
        assert!(effective_duration(Some(10.0), Some(6.0), Some(2.0)).is_err());
        assert!(effective_duration(Some(10.0), None, Some(30.0)).is_err());
        assert!(effective_duration(Some(10.0), Some(10.0), None).is_err());
        assert!(effective_duration(Some(10.0), Some(f64::NAN), None).is_err());
        assert!(effective_duration(Some(10.0), Some(-1.0), None).is_err());
    }

    #[test]
    fn job_keeps_trimmed_length_for_progress() {
        let (d, fs) = files(1, "trim");
        let enc = Arc::new(Fake { running: AtomicUsize::new(0), peak: AtomicUsize::new(0), ms: 20, fail_names: vec![] });
        let q = Queue::new(None, 1, enc, |_| {});
        let mut nj = newjob(&fs[0]);
        nj.duration_secs = Some(10.0);
        nj.start_secs = Some(2.0);
        nj.end_secs = Some(5.0);
        let id = q.add(nj, &Settings::default()).unwrap();
        let j = q.jobs().into_iter().find(|j| j.id == id).unwrap();
        assert_eq!(j.duration_secs, Some(3.0));
        assert_eq!(j.source_size, Some(1));
        let mut bad = newjob(&fs[0]);
        bad.duration_secs = Some(10.0);
        bad.start_secs = Some(8.0);
        bad.end_secs = Some(3.0);
        assert!(q.add(bad, &Settings::default()).is_err());
        q.shutdown();
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn manifest_lists_only_finished_jobs_with_checksums() {
        let mk = |id: u64, state: &str| -> Job {
            serde_json::from_value(serde_json::json!({
                "id": id, "source": "C:/a.mov", "file_name": "a.mov", "duration_secs": 4.0,
                "preset": serde_json::to_value(builtin_presets().remove(0)).unwrap(),
                "dest_dir": null, "template": "{name}", "overwrite": "rename", "state": state, "progress": 100.0,
                "fps": null, "speed": null, "eta_secs": null, "error": null, "output": "C:/o.mp4", "note": "Encoded in software",
                "attempt": 1, "created_at": 0, "temp": null, "source_size": 1000,
                "verify": { "ok": true, "notes": [], "size_bytes": 500, "duration_secs": 4.0, "video_codec": "h264", "audio_codec": "aac", "width": 1280, "height": 720, "sha256": "ab".repeat(32) }
            })).unwrap()
        };
        let m = manifest_json(&[mk(1, "done"), mk(2, "failed"), mk(3, "queued")], "9.9");
        assert_eq!(m["manifest_version"], 1);
        let items = m["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["sha256"].as_str().unwrap().len(), 64);
        assert_eq!(items[0]["verified"], true);
        assert_eq!(items[0]["output_size_bytes"], 500);
        assert!(m["generator"].as_str().unwrap().contains("9.9"));
    }

    #[test]
    fn settings_are_clamped() {
        let s = Settings { max_parallel: 99, template: "".into(), default_dest: Some("relative".into()), ..Settings::default() }.sanitized();
        assert_eq!(s.max_parallel, 4);
        assert_eq!(s.template, "{name}-{preset}");
        assert!(s.default_dest.is_none());
        let f = Settings { favorites: vec!["ok-id".into(), "ok-id".into(), "../bad".into(), "".into(), "x".repeat(70)], ..Settings::default() }.sanitized();
        assert_eq!(f.favorites, vec!["ok-id".to_string()]);
    }
}
