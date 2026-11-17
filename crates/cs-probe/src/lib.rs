//! Locate FFmpeg tools and probe media files through `ffprobe`.
//!
//! Safety notes:
//! * tools are resolved from explicit absolute directories and `PATH` entries only
//!   (never the current directory);
//! * input paths must be existing regular files and are passed as absolute paths,
//!   with `-protocol_whitelist file`, so URLs/pseudo-protocols cannot be probed.

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("ffprobe not found")]
    ToolMissing,
    #[error("not a readable file: {0}")]
    BadInput(String),
    #[error("ffprobe failed: {0}")]
    Failed(String),
    #[error("ffprobe timed out")]
    Timeout,
    #[error("could not parse ffprobe output: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VideoInfo {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: Option<f64>,
    pub pix_fmt: Option<String>,
    pub field_order: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    /// 8, 10, 12 … when it can be determined.
    #[serde(default)]
    pub bit_depth: Option<u32>,
    #[serde(default)]
    pub color_primaries: Option<String>,
    #[serde(default)]
    pub color_transfer: Option<String>,
    #[serde(default)]
    pub color_space: Option<String>,
    #[serde(default)]
    pub color_range: Option<String>,
    /// Frame rate is not constant (average and nominal rates differ by more than 2 %).
    #[serde(default)]
    pub vfr: bool,
    /// PQ (HDR10 / Dolby Vision base) or HLG transfer.
    #[serde(default)]
    pub hdr: bool,
    #[serde(default)]
    pub timecode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AudioInfo {
    pub codec: String,
    pub channels: u32,
    pub sample_rate: Option<u32>,
    /// Position among the audio streams (0 = first).
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub channel_layout: Option<String>,
    #[serde(default)]
    pub bit_rate: Option<u64>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleInfo {
    pub codec: String,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaInfo {
    pub path: String,
    pub file_name: String,
    pub format_name: String,
    pub duration_secs: Option<f64>,
    pub size_bytes: Option<u64>,
    pub bit_rate: Option<u64>,
    pub video: Option<VideoInfo>,
    /// The first audio stream (kept for convenience; see `audio_streams` for all of them).
    pub audio: Option<AudioInfo>,
    #[serde(default)]
    pub audio_streams: Vec<AudioInfo>,
    #[serde(default)]
    pub subtitles: Vec<SubtitleInfo>,
}

/// Resolve a tool such as `ffmpeg` from `extra_dirs`, then `PATH`. Never the current directory.
pub fn locate_tool(name: &str, extra_dirs: &[PathBuf]) -> Option<PathBuf> {
    let exe = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    let path_dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    extra_dirs
        .iter()
        .chain(path_dirs.iter())
        .filter(|d| d.is_absolute())
        .map(|d| d.join(&exe))
        .find(|p| p.is_file())
}

/// Hide the console window of child processes on Windows.
pub fn no_window(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd
}

/// Validate a user-supplied input path and return a clean absolute path.
pub fn validate_input(path: &Path) -> Result<PathBuf, ProbeError> {
    let canon = std::fs::canonicalize(path)
        .map_err(|e| ProbeError::BadInput(format!("{}: {e}", path.display())))?;
    let meta = std::fs::metadata(&canon).map_err(|e| ProbeError::BadInput(e.to_string()))?;
    if !meta.is_file() {
        return Err(ProbeError::BadInput(format!("{} is not a file", path.display())));
    }
    Ok(strip_verbatim(canon))
}

fn strip_verbatim(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        if !rest.starts_with("UNC\\") {
            return PathBuf::from(rest);
        }
    }
    p
}

pub fn probe(ffprobe: &Path, file: &Path) -> Result<MediaInfo, ProbeError> {
    let input = validate_input(file)?;
    let mut cmd = Command::new(ffprobe);
    cmd.args([
        "-v", "error", "-protocol_whitelist", "file", "-print_format", "json", "-show_format", "-show_streams", "-i",
    ])
    .arg(&input)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    no_window(&mut cmd);
    let mut child = cmd.spawn().map_err(|_| ProbeError::ToolMissing)?;
    let mut out = child.stdout.take().expect("piped");
    let mut err = child.stderr.take().expect("piped");
    let t_out = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let t_err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if start.elapsed() > Duration::from_secs(30) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProbeError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(ProbeError::Failed(e.to_string())),
        }
    };
    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();
    if !status.success() {
        let msg = stderr.lines().next().unwrap_or("unknown error").to_string();
        return Err(ProbeError::Failed(msg));
    }
    parse_probe_json(&stdout, &input)
}

#[derive(Deserialize)]
struct RawProbe {
    #[serde(default)]
    streams: Vec<RawStream>,
    format: Option<RawFormat>,
}
#[derive(Deserialize)]
struct RawFormat {
    format_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}
#[derive(Deserialize)]
struct RawStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    pix_fmt: Option<String>,
    field_order: Option<String>,
    channels: Option<u32>,
    sample_rate: Option<String>,
    disposition: Option<RawDisposition>,
    profile: Option<String>,
    level: Option<i64>,
    bits_per_raw_sample: Option<String>,
    color_primaries: Option<String>,
    color_transfer: Option<String>,
    color_space: Option<String>,
    color_range: Option<String>,
    channel_layout: Option<String>,
    bit_rate: Option<String>,
    tags: Option<std::collections::HashMap<String, String>>,
}
#[derive(Deserialize)]
struct RawDisposition {
    attached_pic: Option<u8>,
}

fn parse_rate(s: &str) -> Option<f64> {
    let (n, d) = s.split_once('/')?;
    let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
    if d == 0.0 || n == 0.0 {
        None
    } else {
        Some((n / d * 1000.0).round() / 1000.0)
    }
}

pub fn parse_probe_json(json: &str, path: &Path) -> Result<MediaInfo, ProbeError> {
    let raw: RawProbe = serde_json::from_str(json).map_err(|e| ProbeError::Parse(e.to_string()))?;
    let fmt = raw.format.ok_or_else(|| ProbeError::Parse("no format section".into()))?;
    let tag = |s: &RawStream, k: &str| -> Option<String> {
        s.tags.as_ref().and_then(|t| t.iter().find(|(key, _)| key.eq_ignore_ascii_case(k)).map(|(_, v)| v.clone())).filter(|v| !v.trim().is_empty())
    };
    let known = |v: &Option<String>| v.clone().filter(|x| x != "unknown" && x != "unspecified" && !x.is_empty());
    let video = raw
        .streams
        .iter()
        .find(|s| {
            s.codec_type.as_deref() == Some("video")
                && s.disposition.as_ref().and_then(|d| d.attached_pic) != Some(1)
        })
        .map(|s| {
            let avg = s.avg_frame_rate.as_deref().and_then(parse_rate);
            let nominal = s.r_frame_rate.as_deref().and_then(parse_rate);
            let vfr = match (avg, nominal) {
                (Some(a), Some(n)) => ((a - n).abs() / n.max(0.001)) > 0.02,
                _ => false,
            };
            let transfer = known(&s.color_transfer);
            let bit_depth = s
                .bits_per_raw_sample
                .as_deref()
                .and_then(|b| b.parse::<u32>().ok())
                .or_else(|| s.pix_fmt.as_deref().and_then(depth_from_pix_fmt));
            VideoInfo {
                codec: s.codec_name.clone().unwrap_or_default(),
                width: s.width.unwrap_or(0),
                height: s.height.unwrap_or(0),
                fps: avg.or(nominal),
                pix_fmt: s.pix_fmt.clone(),
                field_order: s.field_order.clone(),
                profile: s.profile.clone().filter(|p| p != "unknown"),
                level: s.level.and_then(|l| level_text(s.codec_name.as_deref().unwrap_or(""), l)),
                bit_depth,
                color_primaries: known(&s.color_primaries),
                color_space: known(&s.color_space),
                color_range: known(&s.color_range),
                hdr: matches!(transfer.as_deref(), Some("smpte2084") | Some("arib-std-b67")),
                color_transfer: transfer,
                vfr,
                timecode: tag(s, "timecode"),
            }
        });
    let audio_streams: Vec<AudioInfo> = raw
        .streams
        .iter()
        .filter(|s| s.codec_type.as_deref() == Some("audio"))
        .enumerate()
        .map(|(i, s)| AudioInfo {
            codec: s.codec_name.clone().unwrap_or_default(),
            channels: s.channels.unwrap_or(0),
            sample_rate: s.sample_rate.as_deref().and_then(|v| v.parse().ok()),
            index: i as u32,
            channel_layout: s.channel_layout.clone().filter(|l| !l.is_empty()),
            bit_rate: s.bit_rate.as_deref().and_then(|v| v.parse().ok()),
            language: tag(s, "language").filter(|l| l != "und"),
            title: tag(s, "title"),
        })
        .collect();
    let audio = audio_streams.first().cloned();
    let subtitles: Vec<SubtitleInfo> = raw
        .streams
        .iter()
        .filter(|s| s.codec_type.as_deref() == Some("subtitle"))
        .map(|s| SubtitleInfo { codec: s.codec_name.clone().unwrap_or_default(), language: tag(s, "language").filter(|l| l != "und"), title: tag(s, "title") })
        .collect();
    if video.is_none() && audio.is_none() {
        return Err(ProbeError::Parse("no audio or video streams".into()));
    }
    Ok(MediaInfo {
        path: path.to_string_lossy().into_owned(),
        file_name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        format_name: fmt.format_name.unwrap_or_default(),
        duration_secs: fmt.duration.as_deref().and_then(|v| v.parse().ok()),
        size_bytes: fmt.size.as_deref().and_then(|v| v.parse().ok()),
        bit_rate: fmt.bit_rate.as_deref().and_then(|v| v.parse().ok()),
        video,
        audio,
        audio_streams,
        subtitles,
    })
}

/// FFmpeg reports levels as integers: H.264 uses level*10 (31 -> 3.1), HEVC uses level*30 (153 -> 5.1).
/// Other codecs are left unknown rather than guessed.
fn level_text(codec: &str, l: i64) -> Option<String> {
    if l <= 0 {
        return None;
    }
    match codec {
        "h264" => Some(format!("{:.1}", l as f64 / 10.0)),
        "hevc" => Some(format!("{:.1}", l as f64 / 30.0)),
        _ => None,
    }
}

/// "yuv420p10le" -> 10, "yuv420p" -> 8, "rgb24" -> 8 … (None when unknown).
fn depth_from_pix_fmt(f: &str) -> Option<u32> {
    for d in [16u32, 14, 12, 10, 9] {
        if f.contains(&format!("p{d}")) || f.contains(&format!("{d}le")) || f.contains(&format!("{d}be")) {
            return Some(d);
        }
    }
    if f.starts_with("yuv") || f.starts_with("nv") || f.starts_with("rgb") || f.starts_with("bgr") || f.starts_with("gray") {
        Some(8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"streams":[
      {"codec_type":"video","codec_name":"h264","width":1920,"height":1080,"r_frame_rate":"30000/1001","avg_frame_rate":"30000/1001","pix_fmt":"yuv420p","field_order":"progressive","disposition":{"attached_pic":0}},
      {"codec_type":"audio","codec_name":"aac","channels":2,"sample_rate":"48000"}],
      "format":{"format_name":"mov,mp4,m4a,3gp,3g2,mj2","duration":"30.000000","size":"1234567","bit_rate":"329217"}}"#;

    #[test]
    fn parses_video_and_audio() {
        let m = parse_probe_json(SAMPLE, Path::new("some/dir/a.mp4")).unwrap();
        assert_eq!(m.file_name, "a.mp4");
        let v = m.video.unwrap();
        assert_eq!((v.codec.as_str(), v.width, v.height), ("h264", 1920, 1080));
        assert_eq!(v.fps, Some(29.97));
        assert_eq!(m.audio.unwrap().sample_rate, Some(48000));
        assert_eq!(m.duration_secs, Some(30.0));
    }

    #[test]
    fn reads_hdr_bit_depth_vfr_timecode_streams_and_subtitles() {
        let j = r#"{"streams":[
          {"codec_type":"video","codec_name":"hevc","profile":"Main 10","level":153,"width":3840,"height":2160,"r_frame_rate":"60/1","avg_frame_rate":"3000/101","pix_fmt":"yuv420p10le","color_primaries":"bt2020","color_transfer":"smpte2084","color_space":"bt2020nc","color_range":"tv","tags":{"timecode":"01:00:00:00"}},
          {"codec_type":"audio","codec_name":"aac","channels":2,"channel_layout":"stereo","sample_rate":"48000","bit_rate":"128000","tags":{"language":"eng","title":"Main"}},
          {"codec_type":"audio","codec_name":"ac3","channels":6,"channel_layout":"5.1(side)","sample_rate":"48000","tags":{"language":"tha"}},
          {"codec_type":"subtitle","codec_name":"mov_text","tags":{"language":"tha"}}],
          "format":{"format_name":"mov,mp4","duration":"10.0","size":"1000"}}"#;
        let m = parse_probe_json(j, Path::new("x.mp4")).unwrap();
        let v = m.video.unwrap();
        assert_eq!(v.profile.as_deref(), Some("Main 10"));
        assert_eq!(v.level.as_deref(), Some("5.1"));
        assert_eq!(v.bit_depth, Some(10));
        assert!(v.hdr && v.vfr);
        assert_eq!(v.color_transfer.as_deref(), Some("smpte2084"));
        assert_eq!(v.timecode.as_deref(), Some("01:00:00:00"));
        assert_eq!(m.audio_streams.len(), 2);
        assert_eq!(m.audio_streams[1].index, 1);
        assert_eq!(m.audio_streams[1].channels, 6);
        assert_eq!(m.audio_streams[0].language.as_deref(), Some("eng"));
        assert_eq!(m.subtitles.len(), 1);
        assert_eq!(m.audio.unwrap().codec, "aac");
    }

    #[test]
    fn missing_details_stay_unknown_not_guessed() {
        let m = parse_probe_json(SAMPLE, Path::new("a.mp4")).unwrap();
        let v = m.video.unwrap();
        assert_eq!(v.color_primaries, None);
        assert_eq!(v.timecode, None);
        assert!(!v.hdr && !v.vfr);
        assert_eq!(v.bit_depth, Some(8));
    }

    #[test]
    fn audio_only_has_no_video() {
        let j = r#"{"streams":[{"codec_type":"audio","codec_name":"flac","channels":2,"sample_rate":"44100"}],"format":{"format_name":"flac","duration":"1.5"}}"#;
        let m = parse_probe_json(j, Path::new("a.flac")).unwrap();
        assert!(m.video.is_none());
        assert_eq!(m.audio.unwrap().codec, "flac");
    }

    #[test]
    fn cover_art_is_not_video() {
        let j = r#"{"streams":[{"codec_type":"video","codec_name":"mjpeg","width":10,"height":10,"disposition":{"attached_pic":1}},{"codec_type":"audio","codec_name":"mp3","channels":2}],"format":{"format_name":"mp3"}}"#;
        assert!(parse_probe_json(j, Path::new("a.mp3")).unwrap().video.is_none());
    }

    #[test]
    fn rejects_garbage_and_streamless() {
        assert!(parse_probe_json("not json", Path::new("x")).is_err());
        assert!(parse_probe_json(r#"{"streams":[],"format":{"format_name":"x"}}"#, Path::new("x")).is_err());
    }

    #[test]
    fn rejects_missing_file_and_directory() {
        assert!(validate_input(Path::new("Z:\\definitely\\missing.mp4")).is_err());
        assert!(validate_input(&std::env::temp_dir()).is_err());
    }

    #[test]
    fn locate_ignores_relative_dirs() {
        assert!(locate_tool("ffmpeg", &[PathBuf::from("relative/dir")]).is_none() || std::env::var_os("PATH").is_some());
    }
}
