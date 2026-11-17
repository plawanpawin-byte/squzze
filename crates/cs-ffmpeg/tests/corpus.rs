//! Test corpus: ~30 deterministic media files generated with FFmpeg's lavfi sources.
//! Rule under test: every file ends as "converted and verified" or "failed with a readable message" — never silent.

use cs_ffmpeg::*;
use cs_probe::{locate_tool, probe};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Expect {
    /// Probes fine and converts + verifies.
    Converts,
    /// ffprobe must reject it with a message.
    ProbeFails,
}

struct Case {
    name: &'static str,
    expect: Expect,
    /// FFmpeg arguments that create the file (output path appended). Empty = custom handling.
    args: Vec<&'static str>,
}

const V720: &str = "testsrc2=size=1280x720:rate=30:duration=1";
const SINE: &str = "sine=frequency=440:duration=1";

fn c(name: &'static str, args: &[&'static str]) -> Case {
    Case { name, expect: Expect::Converts, args: args.to_vec() }
}

fn cases() -> Vec<Case> {
    vec![
        c("h264_720.mp4", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"]),
        c("hevc.mp4", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "libx265", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"]),
        c("vp9.webm", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "libvpx-vp9", "-b:v", "1M", "-c:a", "libopus", "-shortest"]),
        c("av1.mkv", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "libsvtav1", "-c:a", "libopus", "-shortest"]),
        c("prores.mov", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "prores_ks", "-c:a", "pcm_s16le", "-shortest"]),
        c("mjpeg.avi", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "mjpeg", "-c:a", "mp3", "-shortest"]),
        c("mpeg2.ts", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "mpeg2video", "-c:a", "mp2", "-shortest"]),
        c("vertical_1080x1920.mp4", &["-f", "lavfi", "-i", "testsrc2=size=1080x1920:rate=30:duration=1", "-f", "lavfi", "-i", SINE, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"]),
        c("uhd_4k.mp4", &["-f", "lavfi", "-i", "testsrc2=size=3840x2160:rate=24:duration=1", "-c:v", "libx264", "-preset", "ultrafast", "-pix_fmt", "yuv420p"]),
        c("vfr.mp4", &["-f", "lavfi", "-i", "testsrc2=size=640x360:rate=60:duration=2", "-vf", "select='not(mod(n,3))+not(mod(n,7))'", "-fps_mode", "vfr", "-c:v", "libx264", "-pix_fmt", "yuv420p"]),
        c("hdr_tagged_10bit.mkv", &["-f", "lavfi", "-i", V720, "-c:v", "libx265", "-pix_fmt", "yuv420p10le", "-x265-params", "colorprim=bt2020:transfer=smpte2084:colormatrix=bt2020nc:log-level=error"]),
        c("h264_10bit.mp4", &["-f", "lavfi", "-i", V720, "-c:v", "libx264", "-pix_fmt", "yuv420p10le"]),
        c("fps_120.mp4", &["-f", "lavfi", "-i", "testsrc2=size=640x360:rate=120:duration=1", "-c:v", "libx264", "-pix_fmt", "yuv420p"]),
        c("ntsc_29_97.mp4", &["-f", "lavfi", "-i", "testsrc2=size=640x480:rate=30000/1001:duration=1", "-c:v", "libx264", "-pix_fmt", "yuv420p"]),
        c("one_frame.mp4", &["-f", "lavfi", "-i", "testsrc2=size=320x240:rate=30", "-frames:v", "1", "-c:v", "libx264", "-pix_fmt", "yuv420p"]),
        c("no_audio.mp4", &["-f", "lavfi", "-i", V720, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-an"]),
        c("audio_5_1.mkv", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", "sine=frequency=330:duration=1", "-af", "pan=5.1|FL=c0|FR=c0|FC=c0|LFE=c0|BL=c0|BR=c0", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"]),
        c("two_audio_tracks.mkv", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-f", "lavfi", "-i", "sine=frequency=880:duration=1", "-map", "0:v", "-map", "1:a", "-map", "2:a", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"]),
        c("audio_stereo_44k.wav", &["-f", "lavfi", "-i", "sine=frequency=440:duration=1:sample_rate=44100", "-ac", "2"]),
        c("audio_mono_8k.wav", &["-f", "lavfi", "-i", "sine=frequency=440:duration=1:sample_rate=8000", "-ac", "1"]),
        c("audio_96k_24bit.flac", &["-f", "lavfi", "-i", "sine=frequency=440:duration=1:sample_rate=96000", "-sample_fmt", "s32"]),
        c("audio.mp3", &["-f", "lavfi", "-i", SINE, "-c:a", "libmp3lame"]),
        c("audio.opus", &["-f", "lavfi", "-i", SINE, "-c:a", "libopus"]),
        c("audio.m4a", &["-f", "lavfi", "-i", SINE, "-c:a", "aac"]),
        c("ทดสอบ ไฟล์ ชื่อไทย.mp4", &["-f", "lavfi", "-i", V720, "-f", "lavfi", "-i", SINE, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"]),
        c("name with spaces & (parens) [brackets].mp4", &["-f", "lavfi", "-i", V720, "-c:v", "libx264", "-pix_fmt", "yuv420p"]),
        Case { name: "zero_bytes.mp4", expect: Expect::ProbeFails, args: vec![] },
        Case { name: "not_media.txt", expect: Expect::ProbeFails, args: vec![] },
        Case { name: "truncated.mp4", expect: Expect::ProbeFails, args: vec![] },
    ]
}

fn run(ffmpeg: &Path, args: &[&str], out: &Path) -> bool {
    Command::new(ffmpeg)
        .args(["-v", "error", "-y"])
        .args(args)
        .arg(out)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn corpus_every_file_converts_and_verifies_or_fails_readably() {
    let (Some(ffmpeg), Some(ffprobe)) = (locate_tool("ffmpeg", &[]), locate_tool("ffprobe", &[])) else {
        eprintln!("SKIP: ffmpeg not found");
        return;
    };
    let installed = list_encoders(&ffmpeg);
    let dir = std::env::temp_dir().join(format!("sq-corpus-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("out")).unwrap();
    // a deeply nested path (long, but under the classic Windows limit)
    let deep = dir.join("a_long_folder_name_for_path_testing").join("another_quite_long_folder_name").join("and_a_third_one_to_be_sure");
    std::fs::create_dir_all(&deep).unwrap();

    let mut files: Vec<(PathBuf, Expect)> = vec![];
    let mut skipped = vec![];
    for case in cases() {
        let path = dir.join(case.name);
        match case.name {
            "zero_bytes.mp4" => std::fs::write(&path, b"").unwrap(),
            "not_media.txt" => std::fs::write(&path, "hello, this is not a video").unwrap(),
            "truncated.mp4" => {
                let good = dir.join("trunc_src.mp4");
                if run(&ffmpeg, &["-f", "lavfi", "-i", V720, "-c:v", "libx264", "-pix_fmt", "yuv420p"], &good) {
                    let bytes = std::fs::read(&good).unwrap();
                    std::fs::write(&path, &bytes[..bytes.len() / 4]).unwrap();
                } else {
                    skipped.push(case.name);
                    continue;
                }
            }
            _ => {
                if !run(&ffmpeg, &case.args, &path) {
                    skipped.push(case.name); // e.g. the encoder is missing in a reduced FFmpeg build
                    continue;
                }
            }
        }
        files.push((path, case.expect));
    }
    // subtitles: an mp4 with a soft subtitle track
    let srt = dir.join("subs.srt");
    std::fs::write(&srt, "1\n00:00:00,000 --> 00:00:01,000\nสวัสดี Hello\n").unwrap();
    let with_subs = dir.join("with_subtitles.mp4");
    let ok = Command::new(&ffmpeg)
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", V720, "-i"])
        .arg(&srt)
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:s", "mov_text", "-shortest"])
        .arg(&with_subs)
        .stdin(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok { files.push((with_subs, Expect::Converts)) } else { skipped.push("with_subtitles.mp4") }
    // long path
    let deep_file = deep.join("deep_path_clip.mp4");
    if run(&ffmpeg, &["-f", "lavfi", "-i", V720, "-c:v", "libx264", "-pix_fmt", "yuv420p"], &deep_file) {
        files.push((deep_file, Expect::Converts));
    }

    let mut converted = 0;
    let mut rejected = 0;
    let mut problems: Vec<String> = vec![];
    for (path, expect) in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        match (probe(&ffprobe, path), expect) {
            (Err(e), Expect::ProbeFails) => {
                if e.to_string().trim().is_empty() {
                    problems.push(format!("{name}: rejected with an empty message"));
                }
                rejected += 1;
            }
            (Ok(_), Expect::ProbeFails) => problems.push(format!("{name}: should have been rejected")),
            (Err(e), Expect::Converts) => problems.push(format!("{name}: probe failed: {e}")),
            (Ok(info), Expect::Converts) => {
                // the inspector must report these details correctly (or say "unknown"), never guess
                let v = info.video.clone();
                let mut expect = |ok: bool, what: &str| {
                    if !ok {
                        problems.push(format!("{name}: probe detail wrong: {what}"));
                    }
                };
                match name.as_str() {
                    "hdr_tagged_10bit.mkv" => expect(v.as_ref().is_some_and(|v| v.hdr && v.bit_depth == Some(10) && v.color_transfer.as_deref() == Some("smpte2084") && v.color_primaries.as_deref() == Some("bt2020")), "HDR10 tags / 10-bit"),
                    "h264_10bit.mp4" => expect(v.as_ref().is_some_and(|v| v.bit_depth == Some(10) && !v.hdr), "10-bit SDR"),
                    "vfr.mp4" => expect(v.as_ref().is_some_and(|v| v.vfr), "variable frame rate flag"),
                    "h264_720.mp4" => expect(v.as_ref().is_some_and(|v| v.profile.is_some() && v.level.is_some() && v.bit_depth == Some(8) && !v.vfr && !v.hdr), "profile/level/8-bit/CFR"),
                    "two_audio_tracks.mkv" => expect(info.audio_streams.len() == 2 && info.audio_streams[1].index == 1, "two audio streams"),
                    "audio_5_1.mkv" => expect(info.audio.as_ref().is_some_and(|a| a.channels == 6), "5.1 channels"),
                    "audio_mono_8k.wav" => expect(info.audio.as_ref().is_some_and(|a| a.channels == 1 && a.sample_rate == Some(8000)), "mono 8 kHz"),
                    "with_subtitles.mp4" => expect(info.subtitles.len() == 1, "one subtitle stream"),
                    "no_audio.mp4" => expect(info.audio.is_none() && info.audio_streams.is_empty(), "no audio"),
                    _ => {}
                }
                let preset_id = if info.video.is_some() { "h264-720" } else { "aac-256" };
                let preset = builtin_presets().into_iter().find(|p| p.id == preset_id).unwrap();
                if let Some(why) = unavailable_reason(&preset, &installed, &[]) {
                    skipped.push("(encoder missing)");
                    eprintln!("SKIP {name}: {why}");
                    continue;
                }
                let canon = cs_probe::validate_input(path).unwrap();
                let fin = plan_output(&canon, Some(&dir.join("out")), "", &format!("{preset_id}-{converted}"), &preset.container, Overwrite::Rename).unwrap();
                let tmp = temp_path(&fin);
                let args = build_args(&preset, &canon, &tmp, None);
                let (res, log) = run_encode_logged(&ffmpeg, &args, info.duration_secs, &AtomicBool::new(false), |_| {});
                match res {
                    Err(e) => problems.push(format!("{name}: encode failed: {e} / {}", log.last().cloned().unwrap_or_default())),
                    Ok(()) => {
                        std::fs::rename(&tmp, &fin).unwrap();
                        match probe(&ffprobe, &fin) {
                            Err(e) => problems.push(format!("{name}: output unreadable: {e}")),
                            Ok(out) => {
                                let v = verify_output(&out, &preset, info.duration_secs, info.audio.is_some());
                                if v.ok { converted += 1 } else { problems.push(format!("{name}: verify failed: {:?}", v.notes)) }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("corpus: {} files generated, {converted} converted+verified, {rejected} rejected readably, skipped: {skipped:?}", files.len());
    let _ = std::fs::remove_dir_all(&dir);
    assert!(problems.is_empty(), "silent or unexplained failures:\n{}", problems.join("\n"));
    assert!(files.len() >= 25, "corpus too small: {}", files.len());
    assert!(converted >= 18, "too few conversions: {converted}");
    assert!(rejected == 3, "the three bad files must be rejected, got {rejected}");
}
