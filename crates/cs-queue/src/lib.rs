//! Persistent job queue: scheduler, worker threads, cancel/retry and the FFmpeg-backed encoder.

use cs_ffmpeg::{build_args, plan_output, run_encode, temp_path, HwKind, Overwrite, PlanError, Preset, Progress, RunError};
use cs_probe::validate_input;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJob {
    pub source: String,
    pub duration_secs: Option<f64>,
    pub preset: Preset,
    pub dest_dir: Option<String>,
    pub template: Option<String>,
    pub overwrite: Option<Overwrite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub max_parallel: u32,
    pub use_hw: bool,
    pub default_dest: Option<String>,
    pub template: String,
    pub overwrite: Overwrite,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { max_parallel: 1, use_hw: true, default_dest: None, template: "{name}-{preset}".into(), overwrite: Overwrite::Rename }
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
        let mut g = self.shared.inner.lock().unwrap();
        let id = g.next_id;
        g.next_id += 1;
        let job = Job {
            id,
            file_name: src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            source: src.to_string_lossy().into_owned(),
            duration_secs: new.duration_secs,
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

/// Real encoder: runs FFmpeg with temp file + atomic rename, hardware first then software fallback.
pub struct FfmpegEncoder {
    pub ffmpeg: PathBuf,
    pub hw: Arc<Mutex<Vec<HwKind>>>,
    pub settings: Arc<Mutex<Settings>>,
}

fn remove_quiet(p: &Path) {
    let _ = std::fs::remove_file(p);
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
                (Some(v), true) => list.iter().copied().find(|k| k.encoder(v.codec).is_some()),
                _ => None,
            }
        };
        let attempt = |hw: Option<HwKind>, report: &mut dyn FnMut(Report)| {
            let args = build_args(&job.preset, &src, &tmp, hw);
            run_encode(&self.ffmpeg, &args, job.duration_secs, cancel, |p| report(Report::Progress(p)))
        };
        let mut res = attempt(hw_kind, report);
        if let (Err(RunError::Failed(m)), Some(k)) = (&res, hw_kind) {
            remove_quiet(&tmp);
            report(Report::Note(format!("{} failed ({}); retried in software", k.label(), m.chars().take(120).collect::<String>())));
            res = attempt(None, report);
        } else if let (Ok(()), Some(k)) = (&res, hw_kind) {
            report(Report::Note(format!("Encoded with {}", k.label())));
        }
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
        NewJob { source: p.to_string_lossy().into_owned(), duration_secs: Some(1.0), preset: builtin_presets().remove(0), dest_dir: None, template: None, overwrite: None }
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
    fn settings_are_clamped() {
        let s = Settings { max_parallel: 99, template: "".into(), default_dest: Some("relative".into()), ..Settings::default() }.sanitized();
        assert_eq!(s.max_parallel, 4);
        assert_eq!(s.template, "{name}-{preset}");
        assert!(s.default_dest.is_none());
    }
}
