//! Integration tests that run the real FFmpeg. Skipped (with a message) when FFmpeg is absent.

use cs_ffmpeg::*;
use cs_probe::{locate_tool, probe};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

fn tools() -> Option<(PathBuf, PathBuf)> {
    Some((locate_tool("ffmpeg", &[])?, locate_tool("ffprobe", &[])?))
}

fn workdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("squeeze-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn make_fixture(ffmpeg: &Path, dir: &Path, name: &str, secs: u32) -> PathBuf {
    let out = dir.join(name);
    let st = Command::new(ffmpeg)
        .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg(format!("testsrc2=size=1280x720:rate=30:duration={secs}"))
        .args(["-f", "lavfi", "-i"])
        .arg(format!("sine=frequency=440:duration={secs}"))
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"])
        .arg(&out)
        .stdin(Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    out
}

fn encode(ffmpeg: &Path, preset: &Preset, input: &Path, dir: &Path, dur: f64, hw: Option<HwKind>) -> Result<PathBuf, RunError> {
    let fin = plan_output(input, Some(dir), "", &preset.id, &preset.container, Overwrite::Rename).unwrap();
    let tmp = temp_path(&fin);
    let args = build_args(preset, input, &tmp, hw);
    run_encode(ffmpeg, &args, Some(dur), &AtomicBool::new(false), |_| {})?;
    std::fs::rename(&tmp, &fin).unwrap();
    Ok(fin)
}

#[test]
fn every_builtin_preset_encodes_and_probes_correctly() {
    let Some((ffmpeg, ffprobe)) = tools() else {
        eprintln!("SKIP: ffmpeg not found");
        return;
    };
    let dir = workdir("presets");
    let src = make_fixture(&ffmpeg, &dir, "src clip ทดสอบ.mp4", 2);
    for p in builtin_presets() {
        let out = encode(&ffmpeg, &p, &src, &dir, 2.0, None).unwrap_or_else(|e| panic!("{}: {e}", p.id));
        let info = probe(&ffprobe, &out).unwrap_or_else(|e| panic!("{} probe: {e}", p.id));
        let d = info.duration_secs.unwrap_or(0.0);
        assert!((d - 2.0).abs() < 0.35, "{}: duration {d}", p.id);
        match (&p.video, info.video.as_ref()) {
            (Some(v), Some(iv)) => {
                let expect = match v.codec {
                    VideoCodec::H264 => "h264",
                    VideoCodec::Hevc => "hevc",
                    VideoCodec::Av1 => "av1",
                    VideoCodec::Vp9 => "vp9",
                    VideoCodec::Prores422 | VideoCodec::Prores4444 => "prores",
                    VideoCodec::Ffv1 => "ffv1",
                };
                assert_eq!(iv.codec, expect, "{}", p.id);
                if let Some(h) = v.max_height {
                    assert!(iv.height <= h, "{} height {}", p.id, iv.height);
                }
            }
            (None, None) => {}
            other => panic!("{}: video mismatch {other:?}", p.id),
        }
        assert!(info.audio.is_some(), "{} lost audio", p.id);
        assert!(!out.with_file_name(format!("{}.squeeze-part.{}", out.file_stem().unwrap().to_string_lossy(), p.container)).exists());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn progress_is_monotonic_and_reaches_100() {
    let Some((ffmpeg, _)) = tools() else { return };
    let dir = workdir("progress");
    let src = make_fixture(&ffmpeg, &dir, "p.mp4", 6);
    let p = builtin_presets().into_iter().find(|p| p.id == "h264-1080").unwrap();
    let out = dir.join("o.mp4");
    let args = build_args(&p, &src, &out, None);
    let mut seen: Vec<f32> = vec![];
    run_encode(&ffmpeg, &args, Some(6.0), &AtomicBool::new(false), |pr| seen.push(pr.percent)).unwrap();
    assert!(!seen.is_empty());
    assert!(seen.windows(2).all(|w| w[1] >= w[0]), "not monotonic: {seen:?}");
    assert_eq!(*seen.last().unwrap(), 100.0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cancel_stops_process_quickly() {
    let Some((ffmpeg, _)) = tools() else { return };
    let dir = workdir("cancel");
    let src = make_fixture(&ffmpeg, &dir, "c.mp4", 60);
    let p = builtin_presets().into_iter().find(|p| p.id == "av1-1080-webm").unwrap();
    let out = dir.join("o.webm");
    let args = build_args(&p, &src, &out, None);
    let flag = std::sync::Arc::new(AtomicBool::new(false));
    let f2 = flag.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        f2.store(true, Ordering::SeqCst);
    });
    let t = std::time::Instant::now();
    let r = run_encode(&ffmpeg, &args, Some(60.0), &flag, |_| {});
    assert_eq!(r, Err(RunError::Cancelled));
    assert!(t.elapsed().as_secs_f32() < 5.0, "cancel took {:?}", t.elapsed());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn corrupt_input_fails_with_readable_error() {
    let Some((ffmpeg, ffprobe)) = tools() else { return };
    let dir = workdir("bad");
    let bad = dir.join("bad.mp4");
    std::fs::write(&bad, b"this is not a video").unwrap();
    assert!(probe(&ffprobe, &bad).is_err());
    let empty = dir.join("empty.mp4");
    std::fs::write(&empty, b"").unwrap();
    assert!(probe(&ffprobe, &empty).is_err());
    let p = builtin_presets().remove(0);
    let args = build_args(&p, &bad, &dir.join("o.mp4"), None);
    match run_encode(&ffmpeg, &args, None, &AtomicBool::new(false), |_| {}) {
        Err(RunError::Failed(m)) => assert!(!m.is_empty()),
        other => panic!("expected Failed, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn hardware_detection_runs_and_hw_encode_works_if_present() {
    let Some((ffmpeg, _)) = tools() else { return };
    let found = detect_hw(&ffmpeg);
    eprintln!("detected hardware encoders: {found:?}");
    if let Some(hw) = found.first().copied() {
        let dir = workdir("hw");
        let src = make_fixture(&ffmpeg, &dir, "h.mp4", 2);
        let p = builtin_presets().into_iter().find(|p| p.id == "h264-1080").unwrap();
        encode(&ffmpeg, &p, &src, &dir, 2.0, Some(hw)).unwrap_or_else(|e| panic!("{hw:?}: {e}"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
