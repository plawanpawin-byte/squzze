//! Advanced video settings through the real queue + FFmpeg: 10-bit, profile/level, keyframes, two-pass.

use cs_ffmpeg::{builtin_presets, detect_hw, AudioChoice, Preset, RateControl, Speed};
use cs_probe::{locate_tool, probe, MediaInfo};
use cs_queue::{FfmpegEncoder, Job, JobState, NewJob, Queue, Settings};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Env {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    dir: PathBuf,
    src: PathBuf,
    q: Queue,
    settings: Arc<Mutex<Settings>>,
    progress: Arc<Mutex<Vec<f32>>>,
}

fn env(tag: &str) -> Option<Env> {
    let (ffmpeg, ffprobe) = (locate_tool("ffmpeg", &[])?, locate_tool("ffprobe", &[])?);
    let dir = std::env::temp_dir().join(format!("sq-adv-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).ok()?;
    let src = dir.join("in.mp4");
    let ok = Command::new(&ffmpeg)
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=30:duration=4", "-f", "lavfi", "-i", "sine=f=440:duration=4", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"])
        .arg(&src)
        .stdin(Stdio::null())
        .status()
        .ok()?
        .success();
    ok.then_some(())?;
    let hw = detect_hw(&ffmpeg);
    let settings = Arc::new(Mutex::new(Settings::default()));
    let enc = FfmpegEncoder { ffmpeg: ffmpeg.clone(), ffprobe: Some(ffprobe.clone()), hw: Arc::new(Mutex::new(hw)), settings: settings.clone() };
    let progress = Arc::new(Mutex::new(vec![]));
    let p2 = progress.clone();
    let q = Queue::new(None, 1, Arc::new(enc), move |e| {
        if let cs_queue::Event::Updated(j) = e {
            if j.state == JobState::Running {
                p2.lock().unwrap().push(j.progress);
            }
        }
    });
    Some(Env { ffmpeg, ffprobe, dir, src, q, settings, progress })
}

fn run(e: &Env, preset: Preset) -> Job {
    preset.validate().unwrap();
    let id = e
        .q
        .add(
            NewJob { source: e.src.to_string_lossy().into_owned(), duration_secs: Some(4.0), preset, dest_dir: Some(e.dir.join("out").to_string_lossy().into_owned()), template: None, overwrite: None, start_secs: None, end_secs: None, audio: AudioChoice::default(), strip_metadata: None },
            &e.settings.lock().unwrap(),
        )
        .unwrap();
    e.q.start();
    for _ in 0..600 {
        let j = e.q.jobs().into_iter().find(|j| j.id == id).unwrap();
        if matches!(j.state, JobState::Done | JobState::Failed) {
            return j;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("job did not finish");
}

fn preset(id: &str, f: impl FnOnce(&mut cs_ffmpeg::VideoSpec)) -> Preset {
    let mut p = builtin_presets().into_iter().find(|p| p.id == id).unwrap();
    f(p.video.as_mut().unwrap());
    p
}

fn out_info(e: &Env, j: &Job) -> MediaInfo {
    assert_eq!(j.state, JobState::Done, "job failed: {:?}", j.error);
    assert!(j.verify.as_ref().is_some_and(|v| v.ok), "verify: {:?}", j.verify);
    probe(&e.ffprobe, Path::new(j.output.as_ref().unwrap())).unwrap()
}

fn have(e: &Env, enc: &str) -> bool {
    cs_ffmpeg::list_encoders(&e.ffmpeg).contains(enc)
}

#[test]
fn ten_bit_and_profile_level_and_speed() {
    let Some(e) = env("basic") else { return };
    if have(&e, "libx265") {
        let j = run(&e, preset("hevc-1080", |v| { v.bit_depth = Some(10); v.speed = Some(Speed::Fast); }));
        let v = out_info(&e, &j).video.unwrap();
        assert_eq!((v.bit_depth, v.pix_fmt.as_deref()), (Some(10), Some("yuv420p10le")));
        assert!(j.note.as_deref().unwrap_or("").contains("software"), "10-bit must use the software encoder: {:?}", j.note);
    }
    let j = run(&e, preset("h264-720", |v| { v.profile = Some("high".into()); v.level = Some("4.1".into()); v.speed = Some(Speed::Fast); }));
    let v = out_info(&e, &j).video.unwrap();
    assert_eq!(v.profile.as_deref(), Some("High"));
    assert_eq!(v.level.as_deref(), Some("4.1"));
    assert!(j.cmdline.as_deref().unwrap().contains("-profile:v high"));
    assert!(j.note.as_deref().unwrap_or("").contains("advanced settings"), "{:?}", j.note);
    let _ = std::fs::remove_dir_all(&e.dir);
}

#[test]
fn keyframe_interval_is_honoured() {
    let Some(e) = env("gop") else { return };
    let j = run(&e, preset("h264-720", |v| v.keyint_secs = Some(1.0)));
    let out = j.output.clone().unwrap();
    let r = Command::new(&e.ffprobe)
        .args(["-v", "error", "-select_streams", "v:0", "-skip_frame", "nokey", "-show_entries", "frame=pts_time", "-of", "csv=p=0"])
        .arg(&out)
        .output()
        .unwrap();
    let keys = String::from_utf8_lossy(&r.stdout).lines().filter(|l| !l.trim().is_empty()).count();
    assert!(keys >= 4, "a 4 s clip with a 1 s interval should have >= 4 keyframes, found {keys}");
    let _ = std::fs::remove_dir_all(&e.dir);
}

#[test]
fn two_pass_hits_the_target_bitrate_and_cleans_up() {
    let Some(e) = env("2pass") else { return };
    let target = 1500u32;
    let j = run(&e, preset("h264-720", |v| { v.rate = RateControl::Bitrate(target); v.two_pass = true; }));
    let info = out_info(&e, &j);
    let video_kbps = {
        let r = Command::new(&e.ffprobe).args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=bit_rate", "-of", "csv=p=0"]).arg(j.output.as_ref().unwrap()).output().unwrap();
        String::from_utf8_lossy(&r.stdout).trim().parse::<f64>().unwrap_or(0.0) / 1000.0
    };
    assert!((video_kbps - target as f64).abs() / (target as f64) < 0.35, "video bitrate {video_kbps} kbit/s vs target {target}");
    assert!(info.duration_secs.unwrap() > 3.5);
    let cmd = j.cmdline.clone().unwrap();
    assert!(cmd.contains("-pass 1") && cmd.contains("-pass 2"), "both passes recorded: {cmd}");
    // progress runs 0..100 over both passes and never goes backwards
    let seen = e.progress.lock().unwrap().clone();
    assert!(seen.windows(2).all(|w| w[1] >= w[0]), "progress went backwards: {seen:?}");
    assert!(seen.iter().any(|p| *p > 5.0 && *p < 50.0) || seen.len() > 2, "no progress seen: {seen:?}");
    // no pass-log leftovers
    let leftovers: Vec<String> = std::fs::read_dir(e.dir.join("out")).unwrap().flatten().map(|f| f.file_name().to_string_lossy().into_owned()).filter(|n| n.contains(".pass") || n.ends_with(".log") || n.ends_with(".mbtree") || n.contains("squeeze-part")).collect();
    assert!(leftovers.is_empty(), "leftover files: {leftovers:?}");
    let _ = std::fs::remove_dir_all(&e.dir);
}

fn title_of(e: &Env, f: &str) -> Option<String> {
    let r = Command::new(&e.ffprobe).args(["-v", "error", "-show_entries", "format_tags=title", "-of", "csv=p=0"]).arg(f).output().unwrap();
    let t = String::from_utf8_lossy(&r.stdout).trim().to_string();
    (!t.is_empty()).then_some(t)
}

#[test]
fn metadata_policy_and_checksum() {
    let Some(e) = env("meta") else { return };
    // give the source a title tag
    let tagged = e.dir.join("tagged.mp4");
    assert!(Command::new(&e.ffmpeg).args(["-v", "error", "-y", "-i"]).arg(&e.src).args(["-c", "copy", "-metadata", "title=Secret holiday", "-metadata", "comment=GPS 13.7,100.5"]).arg(&tagged).status().unwrap().success());
    assert_eq!(title_of(&e, tagged.to_str().unwrap()).as_deref(), Some("Secret holiday"));
    let go = |strip: Option<bool>| -> Job {
        let id = e
            .q
            .add(
                NewJob { source: tagged.to_string_lossy().into_owned(), duration_secs: Some(4.0), preset: builtin_presets().into_iter().find(|p| p.id == "h264-720").unwrap(), dest_dir: Some(e.dir.join(format!("out-{}", strip.map(|s| s.to_string()).unwrap_or("default".into()))).to_string_lossy().into_owned()), template: None, overwrite: None, start_secs: None, end_secs: None, audio: AudioChoice::default(), strip_metadata: strip },
                &e.settings.lock().unwrap(),
            )
            .unwrap();
        e.q.start();
        for _ in 0..300 {
            let j = e.q.jobs().into_iter().find(|j| j.id == id).unwrap();
            if matches!(j.state, JobState::Done | JobState::Failed) {
                return j;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        panic!("timeout");
    };
    let kept = go(Some(false));
    assert_eq!(kept.state, JobState::Done, "{:?}", kept.error);
    assert_eq!(title_of(&e, kept.output.as_ref().unwrap()).as_deref(), Some("Secret holiday"), "metadata is kept by default");
    let stripped = go(Some(true));
    assert_eq!(stripped.state, JobState::Done, "{:?}", stripped.error);
    assert_eq!(title_of(&e, stripped.output.as_ref().unwrap()), None, "metadata must be removed");
    assert!(stripped.strip_metadata && !kept.strip_metadata);
    // the settings default applies when the job does not say
    e.settings.lock().unwrap().strip_metadata = true;
    let by_default = go(None);
    assert!(by_default.strip_metadata);
    assert_eq!(title_of(&e, by_default.output.as_ref().unwrap()), None);
    // checksum matches an independent computation
    let sha = stripped.verify.as_ref().unwrap().sha256.clone().unwrap();
    assert_eq!(sha.len(), 64);
    assert_eq!(sha, cs_ffmpeg::sha256_file(Path::new(stripped.output.as_ref().unwrap())).unwrap());
    let ps = Command::new("powershell").args(["-NoProfile", "-Command", &format!("(Get-FileHash -Algorithm SHA256 -LiteralPath '{}').Hash.ToLower()", stripped.output.as_ref().unwrap())]).output();
    if let Ok(o) = ps {
        let h = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if h.len() == 64 {
            assert_eq!(h, sha, "must equal Windows' own Get-FileHash");
        }
    }
    let _ = std::fs::remove_dir_all(&e.dir);
}
