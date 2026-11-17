//! Forced GPU failure: the job must still finish in software and say so. Uses the real FFmpeg.

use cs_ffmpeg::{builtin_presets, detect_hw, AudioChoice, HwKind};
use cs_probe::locate_tool;
use cs_queue::{FfmpegEncoder, JobState, NewJob, Queue, Settings};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn broken_gpu_encoder_falls_back_to_software_and_verifies() {
    let (Some(ffmpeg), Some(ffprobe)) = (locate_tool("ffmpeg", &[]), locate_tool("ffprobe", &[])) else {
        eprintln!("SKIP: ffmpeg not found");
        return;
    };
    // pick a GPU family that does NOT work on this machine, so the first attempt must fail
    let working = detect_hw(&ffmpeg);
    let Some(broken) = [HwKind::Qsv, HwKind::Amf, HwKind::Nvenc].into_iter().find(|k| !working.contains(k)) else {
        eprintln!("SKIP: every GPU family works here");
        return;
    };
    let dir = std::env::temp_dir().join(format!("sq-fallback-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("in.mp4");
    let ok = Command::new(&ffmpeg)
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=30:duration=2", "-f", "lavfi", "-i", "sine=f=440:duration=2", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"])
        .arg(&src)
        .stdin(Stdio::null())
        .status()
        .unwrap()
        .success();
    assert!(ok);

    let settings = Arc::new(Mutex::new(Settings::default())); // use_hw = true
    let enc = FfmpegEncoder { ffmpeg, ffprobe: Some(ffprobe), hw: Arc::new(Mutex::new(vec![broken])), settings: settings.clone() };
    let q = Queue::new(None, 1, Arc::new(enc), |_| {});
    let preset = builtin_presets().into_iter().find(|p| p.id == "h264-720").unwrap();
    let id = q
        .add(
            NewJob { source: src.to_string_lossy().into_owned(), duration_secs: Some(2.0), preset, dest_dir: Some(dir.join("out").to_string_lossy().into_owned()), template: None, overwrite: None, start_secs: None, end_secs: None, audio: AudioChoice::default(), strip_metadata: None },
            &settings.lock().unwrap(),
        )
        .unwrap();
    q.start();
    let mut job = None;
    for _ in 0..240 {
        let j = q.jobs().into_iter().find(|j| j.id == id).unwrap();
        if matches!(j.state, JobState::Done | JobState::Failed) {
            job = Some(j);
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let job = job.expect("job did not finish");
    assert_eq!(job.state, JobState::Done, "error: {:?}", job.error);
    let note = job.note.clone().unwrap_or_default();
    assert!(note.contains("software"), "note should say it fell back: {note}");
    assert!(job.verify.as_ref().is_some_and(|v| v.ok), "verify: {:?}", job.verify);
    assert!(job.cmdline.as_deref().unwrap_or("").contains("libx264"), "final command must be the software one");
    eprintln!("fallback ok with broken={broken:?}; note: {note}");
    q.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
}
