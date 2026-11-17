//! Squeeze desktop shell: thin, validated bridge between the UI and the Rust engines.

use base64::Engine as _;
use cs_ffmpeg::{builtin_presets, detect_hw, ffmpeg_version, list_encoders, preview_frame, unavailable_reason, HwKind, Preset, RunError};
use cs_probe::{locate_tool, probe, validate_input, MediaInfo};
use cs_queue::{write_atomic, Encoder, Event, FfmpegEncoder, Job, NewJob, Queue, Report, Settings};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};
use tauri_plugin_dialog::DialogExt;

const MAX_JOBS: usize = 5000;
const MAX_PROBE_BATCH: usize = 300;
const MEDIA_EXT: &[&str] = &[
    "mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "flv", "mpg", "mpeg", "ts", "mts", "m2ts", "mxf", "ogv", "3gp", "mp3", "m4a", "aac", "wav", "flac", "ogg", "opus", "aiff", "wma",
];

struct MissingEncoder;
impl Encoder for MissingEncoder {
    fn encode(&self, _: &Job, _: &AtomicBool, _: &mut dyn FnMut(Report)) -> Result<String, RunError> {
        Err(RunError::Failed(
            "FFmpeg was not found. Install it (for example: winget install Gyan.FFmpeg) or put ffmpeg.exe and ffprobe.exe in the 'ffmpeg' folder next to Squeeze, then restart.".into(),
        ))
    }
}

struct AppState {
    queue: Queue,
    settings: Arc<Mutex<Settings>>,
    hw: Arc<Mutex<Vec<HwKind>>>,
    hw_ready: Arc<AtomicBool>,
    encoders: HashSet<String>,
    ffmpeg: Option<PathBuf>,
    ffprobe: Option<PathBuf>,
    dir: PathBuf,
    custom: Mutex<Vec<Preset>>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Ev {
    Updated { job: Box<Job> },
    Removed { id: u64 },
    Idle,
}

#[derive(Serialize)]
struct AppInfo {
    version: String,
    ffmpeg_path: Option<String>,
    ffmpeg_version: Option<String>,
    hw: Vec<String>,
    hw_ready: bool,
}

#[derive(Serialize)]
struct ProbeResult {
    path: String,
    info: Option<MediaInfo>,
    error: Option<String>,
}

fn all_presets(s: &AppState) -> Vec<Preset> {
    let mut v = builtin_presets();
    v.extend(s.custom.lock().unwrap().iter().cloned());
    v
}

/// id -> reason, for presets the installed FFmpeg cannot run (empty when everything works).
fn availability(s: &AppState) -> HashMap<String, String> {
    let hw = s.hw.lock().unwrap().clone();
    all_presets(s).into_iter().filter_map(|p| unavailable_reason(&p, &s.encoders, &hw).map(|why| (p.id, why))).collect()
}

#[tauri::command]
fn preset_availability(state: State<'_, AppState>) -> HashMap<String, String> {
    availability(&state)
}

fn save_custom(s: &AppState) -> Result<(), String> {
    let json = serde_json::to_vec_pretty(&*s.custom.lock().unwrap()).map_err(|e| e.to_string())?;
    write_atomic(&s.dir.join("presets.json"), &json).map_err(|e| e.to_string())
}

fn check_dest(d: &str) -> Result<(), String> {
    let p = Path::new(d);
    if !p.is_absolute() {
        return Err("destination must be an absolute folder path".into());
    }
    if p.exists() && !p.is_dir() {
        return Err("destination is not a folder".into());
    }
    Ok(())
}

#[tauri::command]
fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        ffmpeg_path: state.ffmpeg.as_ref().map(|p| p.to_string_lossy().into_owned()),
        ffmpeg_version: state.ffmpeg.as_deref().and_then(ffmpeg_version),
        hw: state.hw.lock().unwrap().iter().map(|h| h.label().to_string()).collect(),
        hw_ready: state.hw_ready.load(Ordering::SeqCst),
    }
}

#[tauri::command]
fn list_presets(state: State<'_, AppState>) -> Vec<Preset> {
    all_presets(&state)
}

#[tauri::command]
fn save_custom_preset(state: State<'_, AppState>, mut preset: Preset) -> Result<Vec<Preset>, String> {
    preset.group = "Custom".into();
    preset.validate().map_err(|e| e.to_string())?;
    if builtin_presets().iter().any(|b| b.id == preset.id) {
        return Err("that id belongs to a built-in preset".into());
    }
    {
        let mut c = state.custom.lock().unwrap();
        if let Some(slot) = c.iter_mut().find(|p| p.id == preset.id) {
            *slot = preset;
        } else if c.len() >= 200 {
            return Err("too many custom presets".into());
        } else {
            c.push(preset);
        }
    }
    save_custom(&state)?;
    Ok(all_presets(&state))
}

#[tauri::command]
fn delete_custom_preset(state: State<'_, AppState>, id: String) -> Result<Vec<Preset>, String> {
    state.custom.lock().unwrap().retain(|p| p.id != id);
    save_custom(&state)?;
    Ok(all_presets(&state))
}

#[tauri::command]
async fn export_preset(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let preset = all_presets(&state).into_iter().find(|p| p.id == id).ok_or("unknown preset")?;
    let json = serde_json::to_vec_pretty(&preset).map_err(|e| e.to_string())?;
    let app2 = app.clone();
    let name = format!("{}.squeeze-preset.json", preset.id);
    let chosen = tauri::async_runtime::spawn_blocking(move || app2.dialog().file().set_file_name(name).add_filter("Preset", &["json"]).blocking_save_file())
        .await
        .map_err(|e| e.to_string())?;
    let Some(path) = chosen.and_then(|p| p.into_path().ok()) else { return Ok(false) };
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn import_preset(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Preset>, String> {
    let app2 = app.clone();
    let chosen = tauri::async_runtime::spawn_blocking(move || app2.dialog().file().add_filter("Preset", &["json"]).blocking_pick_file())
        .await
        .map_err(|e| e.to_string())?;
    let Some(path) = chosen.and_then(|p| p.into_path().ok()) else { return Ok(all_presets(&state)) };
    if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(u64::MAX) > 64 * 1024 {
        return Err("preset file is too large".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let preset = Preset::from_json(&text).map_err(|e| e.to_string())?;
    save_custom_preset(state, preset)
}

#[tauri::command]
async fn pick_files(app: AppHandle) -> Result<Vec<String>, String> {
    let r = tauri::async_runtime::spawn_blocking(move || app.dialog().file().add_filter("Media files", MEDIA_EXT).add_filter("All files", &["*"]).blocking_pick_files())
        .await
        .map_err(|e| e.to_string())?;
    Ok(r.unwrap_or_default().into_iter().filter_map(|p| p.into_path().ok()).map(|p| p.to_string_lossy().into_owned()).collect())
}

#[tauri::command]
async fn pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    let r = tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder()).await.map_err(|e| e.to_string())?;
    Ok(r.and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
async fn probe_files(state: State<'_, AppState>, paths: Vec<String>) -> Result<Vec<ProbeResult>, String> {
    if paths.len() > MAX_PROBE_BATCH {
        return Err(format!("too many files at once (max {MAX_PROBE_BATCH})"));
    }
    let ffprobe = state.ffprobe.clone();
    tauri::async_runtime::spawn_blocking(move || {
        paths
            .into_iter()
            .map(|p| match &ffprobe {
                None => ProbeResult { path: p, info: None, error: Some("ffprobe not found".into()) },
                Some(tool) => match probe(tool, Path::new(&p)) {
                    Ok(info) => ProbeResult { path: p, info: Some(info), error: None },
                    Err(e) => ProbeResult { path: p, info: None, error: Some(e.to_string()) },
                },
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview(state: State<'_, AppState>, path: String, secs: f64) -> Result<String, String> {
    let ffmpeg = state.ffmpeg.clone().ok_or("ffmpeg not found")?;
    tauri::async_runtime::spawn_blocking(move || {
        let input = validate_input(Path::new(&path)).map_err(|e| e.to_string())?;
        let bytes = preview_frame(&ffmpeg, &input, secs).map_err(|e| e.to_string())?;
        Ok(format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn add_job(
    state: State<'_, AppState>,
    source: String,
    preset_id: String,
    dest_dir: Option<String>,
    start_secs: Option<f64>,
    end_secs: Option<f64>,
) -> Result<u64, String> {
    if state.queue.jobs().len() >= MAX_JOBS {
        return Err("queue is full".into());
    }
    if let Some(d) = &dest_dir {
        check_dest(d)?;
    }
    let preset = all_presets(&state).into_iter().find(|p| p.id == preset_id).ok_or("unknown preset")?;
    if let Some(why) = availability(&state).remove(&preset.id) {
        return Err(why);
    }
    let ffprobe = state.ffprobe.clone();
    let src = source.clone();
    let duration = tauri::async_runtime::spawn_blocking(move || ffprobe.and_then(|t| probe(&t, Path::new(&src)).ok()).and_then(|i| i.duration_secs))
        .await
        .map_err(|e| e.to_string())?;
    let settings = state.settings.lock().unwrap().clone();
    state.queue.add(NewJob { source, duration_secs: duration, preset, dest_dir, template: None, overwrite: None, start_secs, end_secs }, &settings)
}

#[tauri::command]
fn queue_list(state: State<'_, AppState>) -> Vec<Job> {
    state.queue.jobs()
}
#[tauri::command]
fn queue_start(state: State<'_, AppState>) {
    state.queue.start()
}
#[tauri::command]
fn queue_pause(state: State<'_, AppState>) {
    state.queue.pause()
}
#[tauri::command]
fn job_cancel(state: State<'_, AppState>, id: u64) {
    state.queue.cancel(id)
}
#[tauri::command]
fn queue_cancel_all(state: State<'_, AppState>) {
    state.queue.cancel_all()
}
#[tauri::command]
fn job_retry(state: State<'_, AppState>, id: u64) {
    state.queue.retry(id)
}
#[tauri::command]
fn job_remove(state: State<'_, AppState>, id: u64) -> bool {
    state.queue.remove(id)
}
#[tauri::command]
fn queue_clear_finished(state: State<'_, AppState>) {
    state.queue.clear_finished()
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn set_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    if let Some(d) = &settings.default_dest {
        check_dest(d)?;
    }
    let s = settings.sanitized();
    *state.settings.lock().unwrap() = s.clone();
    state.queue.set_max_parallel(s.max_parallel);
    let json = serde_json::to_vec_pretty(&s).map_err(|e| e.to_string())?;
    write_atomic(&state.dir.join("settings.json"), &json).map_err(|e| e.to_string())?;
    Ok(s)
}

/// Show a finished output in the file manager. Only paths of known finished jobs are accepted.
#[tauri::command]
fn reveal_output(state: State<'_, AppState>, id: u64) -> Result<(), String> {
    let job = state.queue.jobs().into_iter().find(|j| j.id == id).ok_or("unknown job")?;
    let out = job.output.ok_or("job has no output")?;
    if !Path::new(&out).exists() {
        return Err("output file no longer exists".into());
    }
    tauri_plugin_opener::reveal_item_in_dir(out).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let settings: Settings = std::fs::read_to_string(dir.join("settings.json"))
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_default();
            let settings = Arc::new(Mutex::new(settings.sanitized()));
            let custom: Vec<Preset> = std::fs::read_to_string(dir.join("presets.json"))
                .ok()
                .and_then(|t| serde_json::from_str::<Vec<Preset>>(&t).ok())
                .map(|v| v.into_iter().filter(|p| p.validate().is_ok()).collect())
                .unwrap_or_default();

            let mut extra: Vec<PathBuf> = vec![];
            if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
                extra.push(exe_dir.join("ffmpeg"));
                extra.push(exe_dir);
            }
            let ffmpeg = locate_tool("ffmpeg", &extra);
            let ffprobe = locate_tool("ffprobe", &extra);
            #[allow(unused_mut)]
            let mut encoders: HashSet<String> = ffmpeg.as_deref().map(list_encoders).unwrap_or_default();
            // Test hook, debug builds only: pretend some encoders are missing (used by scripts/e2e-availability.mjs).
            #[cfg(debug_assertions)]
            if let Ok(v) = std::env::var("SQUEEZE_TEST_HIDE_ENCODERS") {
                for n in v.split(',') {
                    encoders.remove(n.trim());
                }
            }
            let hw: Arc<Mutex<Vec<HwKind>>> = Arc::new(Mutex::new(vec![]));
            let hw_ready = Arc::new(AtomicBool::new(false));

            let encoder: Arc<dyn Encoder> = match &ffmpeg {
                Some(f) => Arc::new(FfmpegEncoder { ffmpeg: f.clone(), ffprobe: ffprobe.clone(), hw: hw.clone(), settings: settings.clone() }),
                None => Arc::new(MissingEncoder),
            };
            let handle = app.handle().clone();
            let max_parallel = settings.lock().unwrap().max_parallel;
            let queue = Queue::new(Some(dir.join("queue.json")), max_parallel, encoder, move |e| {
                let ev = match e {
                    Event::Updated(j) => Ev::Updated { job: j },
                    Event::Removed(id) => Ev::Removed { id },
                    Event::Idle => Ev::Idle,
                };
                let _ = handle.emit("queue", ev);
            });

            if let Some(f) = ffmpeg.clone() {
                let (hw2, ready2, h2) = (hw.clone(), hw_ready.clone(), app.handle().clone());
                std::thread::spawn(move || {
                    *hw2.lock().unwrap() = detect_hw(&f);
                    ready2.store(true, Ordering::SeqCst);
                    let _ = h2.emit("hw-ready", ());
                });
            } else {
                hw_ready.store(true, Ordering::SeqCst);
            }

            app.manage(AppState { queue, settings, hw, hw_ready, encoders, ffmpeg, ffprobe, dir, custom: Mutex::new(custom) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info, list_presets, preset_availability, save_custom_preset, delete_custom_preset, export_preset, import_preset, pick_files, pick_folder, probe_files, preview, add_job,
            queue_list, queue_start, queue_pause, job_cancel, queue_cancel_all, job_retry, job_remove, queue_clear_finished, get_settings, set_settings, reveal_output
        ])
        .build(tauri::generate_context!())
        .expect("error while building Squeeze");
    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            if let Some(s) = handle.try_state::<AppState>() {
                s.queue.shutdown();
            }
        }
    });
}
