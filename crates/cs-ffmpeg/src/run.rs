//! Run FFmpeg: progress parsing, cancellation, process cleanup, hardware detection.

use crate::args::HwKind;
use cs_probe::no_window;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Progress {
    /// 0.0 – 100.0
    pub percent: f32,
    pub fps: Option<f64>,
    pub speed: Option<f64>,
    pub eta_secs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RunError {
    #[error("could not start ffmpeg: {0}")]
    Spawn(String),
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Failed(String),
}

#[cfg(windows)]
mod job {
    //! One process-wide Job Object that kills every child when Squeeze exits or crashes.
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::*;

    static JOB: OnceLock<usize> = OnceLock::new();

    fn job() -> usize {
        *JOB.get_or_init(|| unsafe {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if h.is_null() {
                return 0;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            h as usize
        })
    }

    pub fn assign(child: &std::process::Child) {
        use std::os::windows::io::AsRawHandle;
        let j = job();
        if j != 0 {
            unsafe {
                AssignProcessToJobObject(j as HANDLE, child.as_raw_handle() as HANDLE);
            }
        }
    }
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
    line.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
}

/// Run `ffmpeg` with `args` (which must include `-progress pipe:1`).
/// `duration` (seconds) is used to turn output time into a percentage.
pub fn run_encode(
    ffmpeg: &Path,
    args: &[String],
    duration: Option<f64>,
    cancel: &AtomicBool,
    on_progress: impl FnMut(Progress),
) -> Result<(), RunError> {
    run_encode_logged(ffmpeg, args, duration, cancel, on_progress).0
}

/// Like [`run_encode`] but also returns the last lines FFmpeg wrote to stderr (success or failure).
pub fn run_encode_logged(
    ffmpeg: &Path,
    args: &[String],
    duration: Option<f64>,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(Progress),
) -> (Result<(), RunError>, Vec<String>) {
    let mut tail_out = Vec::new();
    let r = run_inner(ffmpeg, args, duration, cancel, &mut on_progress, &mut tail_out);
    (r, tail_out)
}

fn run_inner(
    ffmpeg: &Path,
    args: &[String],
    duration: Option<f64>,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(Progress),
    log_out: &mut Vec<String>,
) -> Result<(), RunError> {
    let mut cmd = Command::new(ffmpeg);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    no_window(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| RunError::Spawn(e.to_string()))?;
    #[cfg(windows)]
    job::assign(&child);

    let stdout = child.stdout.take().expect("piped");
    let stderr = child.stderr.take().expect("piped");
    let tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let tail2 = tail.clone();
    let t_err = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let mut t = tail2.lock().unwrap();
            if t.len() >= 30 {
                t.pop_front();
            }
            t.push_back(line.chars().take(400).collect());
        }
    });
    let (tx, rx) = mpsc::channel::<String>();
    let t_out = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    let mut cur = Progress::default();
    let mut out_time: f64 = 0.0;
    let mut last_emit = Instant::now() - Duration::from_secs(1);
    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = (t_out.join(), t_err.join());
            log_out.extend(tail.lock().unwrap().iter().cloned());
            return Err(RunError::Cancelled);
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                if let Some((k, v)) = parse_kv(&line) {
                    match k {
                        "out_time_us" | "out_time_ms" => {
                            if let Ok(us) = v.parse::<f64>() {
                                // never go backwards: FFmpeg can briefly report an earlier time while flushing
                                out_time = out_time.max((us / 1_000_000.0).max(0.0));
                            }
                        }
                        "fps" => cur.fps = v.parse().ok(),
                        "speed" => cur.speed = v.trim_end_matches('x').trim().parse().ok(),
                        "progress" => {
                            let end = v == "end";
                            if let Some(d) = duration.filter(|d| *d > 0.0) {
                                let p = if end { 100.0 } else { ((out_time / d) * 100.0).clamp(0.0, 99.9) as f32 };
                                cur.percent = cur.percent.max(p);
                                cur.eta_secs = cur.speed.filter(|s| *s > 0.0).map(|s| ((d - out_time) / s).max(0.0));
                            } else if end {
                                cur.percent = 100.0;
                            }
                            if end || last_emit.elapsed() >= Duration::from_millis(250) {
                                on_progress(cur.clone());
                                last_emit = Instant::now();
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    let status = child.wait().map_err(|e| RunError::Failed(e.to_string()))?;
    let _ = (t_out.join(), t_err.join());
    log_out.extend(tail.lock().unwrap().iter().cloned());
    if status.success() {
        Ok(())
    } else {
        let t = tail.lock().unwrap();
        let msg: Vec<&str> = t.iter().rev().take(4).map(String::as_str).collect();
        let msg = msg.into_iter().rev().collect::<Vec<_>>().join(" | ");
        Err(RunError::Failed(if msg.is_empty() { format!("ffmpeg exited with {status}") } else { msg }))
    }
}

/// `ffmpeg -version` first line, or None if the binary does not run.
pub fn ffmpeg_version(ffmpeg: &Path) -> Option<String> {
    let mut cmd = Command::new(ffmpeg);
    cmd.arg("-version").stdin(Stdio::null()).stderr(Stdio::null());
    no_window(&mut cmd);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).lines().next().map(str::to_string)
}

fn test_encode(ffmpeg: &Path, enc_args: &[&str]) -> bool {
    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-v", "error", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=30", "-frames:v", "5"])
        .args(enc_args)
        .args(["-f", "null", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    no_window(&mut cmd);
    let Ok(mut child) = cmd.spawn() else { return false };
    #[cfg(windows)]
    job::assign(&child);
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) => return st.success(),
            Ok(None) if start.elapsed() > Duration::from_secs(20) => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(30)),
            Err(_) => return false,
        }
    }
}

/// Find which hardware H.264 encoders actually work on this machine (1-frame style test encodes).
pub fn detect_hw(ffmpeg: &Path) -> Vec<HwKind> {
    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-hide_banner", "-encoders"]).stdin(Stdio::null()).stderr(Stdio::null());
    no_window(&mut cmd);
    let Ok(out) = cmd.output() else { return vec![] };
    let list = String::from_utf8_lossy(&out.stdout).to_string();
    let mut found = vec![];
    let candidates: [(HwKind, &str, &[&str]); 3] = [
        (HwKind::Nvenc, "h264_nvenc", &["-c:v", "h264_nvenc", "-rc", "vbr", "-cq", "28", "-b:v", "0"]),
        (HwKind::Amf, "h264_amf", &["-c:v", "h264_amf", "-rc", "cqp", "-qp_i", "28", "-qp_p", "28"]),
        (HwKind::Qsv, "h264_qsv", &["-c:v", "h264_qsv", "-global_quality", "28"]),
    ];
    for (kind, name, args) in candidates {
        if list.contains(name) && test_encode(ffmpeg, args) {
            found.push(kind);
        }
    }
    found
}

/// Extract one JPEG frame (max 640 px wide) at `secs`. Returns the JPEG bytes (≤ 4 MB).
pub fn preview_frame(ffmpeg: &Path, input: &Path, secs: f64) -> Result<Vec<u8>, RunError> {
    use std::io::Read;
    let secs = if secs.is_finite() && secs >= 0.0 { secs } else { 0.0 };
    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-v", "error", "-protocol_whitelist", "file", "-ss"])
        .arg(format!("{secs:.3}"))
        .arg("-i")
        .arg(input)
        .args(["-frames:v", "1", "-an", "-sn", "-vf", "scale='min(640,iw)':-2", "-f", "mjpeg", "-q:v", "4", "pipe:1"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    no_window(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| RunError::Spawn(e.to_string()))?;
    #[cfg(windows)]
    job::assign(&child);
    let mut out = child.stdout.take().expect("piped");
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out.by_ref().take(4 * 1024 * 1024).read_to_end(&mut buf);
        buf
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > Duration::from_secs(15) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(RunError::Failed("preview timed out".into()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(RunError::Failed(e.to_string())),
        }
    }
    let bytes = reader.join().unwrap_or_default();
    if bytes.is_empty() {
        Err(RunError::Failed("no frame".into()))
    } else {
        Ok(bytes)
    }
}
