//! Turn a validated preset into an FFmpeg argument vector (never a shell string).

use crate::audio::{AudioChoice, AudioMode};
use crate::preset::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HwKind {
    Nvenc,
    Amf,
    Qsv,
}

impl HwKind {
    pub fn label(self) -> &'static str {
        match self {
            HwKind::Nvenc => "NVIDIA NVENC",
            HwKind::Amf => "AMD AMF",
            HwKind::Qsv => "Intel Quick Sync",
        }
    }
    /// FFmpeg encoder name for a codec, if this hardware family supports it.
    pub fn encoder(self, c: VideoCodec) -> Option<&'static str> {
        use VideoCodec::*;
        match (self, c) {
            (HwKind::Nvenc, H264) => Some("h264_nvenc"),
            (HwKind::Nvenc, Hevc) => Some("hevc_nvenc"),
            (HwKind::Nvenc, Av1) => Some("av1_nvenc"),
            (HwKind::Amf, H264) => Some("h264_amf"),
            (HwKind::Amf, Hevc) => Some("hevc_amf"),
            (HwKind::Qsv, H264) => Some("h264_qsv"),
            (HwKind::Qsv, Hevc) => Some("hevc_qsv"),
            _ => None,
        }
    }
}

fn s(x: impl Into<String>) -> String {
    x.into()
}

fn x26x_preset(sp: Option<Speed>) -> &'static str {
    match sp {
        Some(Speed::Fast) => "fast",
        Some(Speed::Slow) => "slow",
        _ => "medium",
    }
}

fn pix_fmt(v: &VideoSpec) -> &'static str {
    if v.bit_depth == Some(10) { "yuv420p10le" } else { "yuv420p" }
}

fn video_args(v: &VideoSpec, container: &str, hw: Option<HwKind>, out: &mut Vec<String>) {
    use VideoCodec::*;
    let hw_enc = hw.and_then(|h| h.encoder(v.codec).map(|e| (h, e)));
    match (hw_enc, v.codec) {
        (Some((kind, enc)), _) => {
            out.extend([s("-c:v"), s(enc)]);
            match (kind, v.rate) {
                (HwKind::Nvenc, RateControl::Crf(q)) => out.extend([s("-rc"), s("vbr"), s("-cq"), q.to_string(), s("-b:v"), s("0")]),
                (HwKind::Amf, RateControl::Crf(q)) => {
                    out.extend([s("-rc"), s("cqp"), s("-qp_i"), q.to_string(), s("-qp_p"), q.to_string()])
                }
                (HwKind::Qsv, RateControl::Crf(q)) => out.extend([s("-global_quality"), q.to_string()]),
                (_, RateControl::Bitrate(k)) => out.extend([s("-b:v"), format!("{k}k")]),
            }
            if matches!(v.codec, Hevc) && (container == "mp4" || container == "mov") {
                out.extend([s("-tag:v"), s("hvc1")]);
            }
        }
        (None, H264) => {
            out.extend([s("-c:v"), s("libx264"), s("-preset"), s(x26x_preset(v.speed))]);
            if let Some(p) = &v.profile {
                out.extend([s("-profile:v"), p.clone()]);
            }
            if let Some(l) = &v.level {
                out.extend([s("-level"), l.clone()]);
            }
            rate_args(v.rate, out);
        }
        (None, Hevc) => {
            out.extend([s("-c:v"), s("libx265"), s("-preset"), s(x26x_preset(v.speed)), s("-x265-params"), s("log-level=error")]);
            rate_args(v.rate, out);
            if container == "mp4" || container == "mov" {
                out.extend([s("-tag:v"), s("hvc1")]);
            }
        }
        (None, Av1) => {
            let n = match v.speed {
                Some(Speed::Fast) => "10",
                Some(Speed::Slow) => "5",
                _ => "8",
            };
            out.extend([s("-c:v"), s("libsvtav1"), s("-preset"), s(n)]);
            rate_args(v.rate, out);
        }
        (None, Vp9) => {
            let cpu = match v.speed {
                Some(Speed::Fast) => "4",
                Some(Speed::Slow) => "1",
                _ => "2",
            };
            out.extend([s("-c:v"), s("libvpx-vp9"), s("-row-mt"), s("1"), s("-deadline"), s("good"), s("-cpu-used"), s(cpu)]);
            if v.bit_depth == Some(10) {
                out.extend([s("-profile:v"), s("2")]);
            }
            match v.rate {
                RateControl::Crf(q) => out.extend([s("-crf"), q.to_string(), s("-b:v"), s("0")]),
                RateControl::Bitrate(k) => out.extend([s("-b:v"), format!("{k}k")]),
            }
        }
        (None, Prores422) => out.extend([s("-c:v"), s("prores_ks"), s("-profile:v"), s("2"), s("-pix_fmt"), s("yuv422p10le")]),
        (None, Prores4444) => out.extend([s("-c:v"), s("prores_ks"), s("-profile:v"), s("4"), s("-pix_fmt"), s("yuv444p10le")]),
        (None, Ffv1) => out.extend([s("-c:v"), s("ffv1"), s("-level"), s("3")]),
    }
    if matches!(v.codec, H264 | Hevc | Av1 | Vp9) {
        out.extend([s("-pix_fmt"), s(pix_fmt(v))]);
    }
    if let Some(k) = v.keyint_secs {
        out.extend([s("-force_key_frames"), format!("expr:gte(t,n_forced*{k})")]);
    }
}

fn rate_args(r: RateControl, out: &mut Vec<String>) {
    match r {
        RateControl::Crf(q) => out.extend([s("-crf"), q.to_string()]),
        RateControl::Bitrate(k) => out.extend([s("-b:v"), format!("{k}k")]),
    }
}

fn audio_args(a: &AudioSpec, channels_override: Option<u32>, out: &mut Vec<String>) {
    let (enc, wants_rate) = match a.codec {
        AudioCodec::Aac => ("aac", true),
        AudioCodec::Opus => ("libopus", true),
        AudioCodec::Mp3 => ("libmp3lame", true),
        AudioCodec::Flac => ("flac", false),
        AudioCodec::Pcm24 => ("pcm_s24le", false),
    };
    out.extend([s("-c:a"), s(enc)]);
    if let (true, Some(k)) = (wants_rate, a.bitrate_kbps) {
        out.extend([s("-b:a"), format!("{k}k")]);
    }
    if let Some(c) = channels_override.or(a.channels) {
        out.extend([s("-ac"), c.to_string()]);
    }
}

/// Metadata policy "strip": do not copy tags (title, location, camera…) or chapters from the source.
pub fn strip_metadata(args: &mut Vec<String>) {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == "-map_metadata" {
            args[i + 1] = s("-1");
        }
    }
    let at = args.iter().position(|x| x == "-map_metadata").map(|i| i + 2).unwrap_or(args.len());
    args.splice(at..at, [s("-map_chapters"), s("-1")]);
}

/// Optional in/out points (seconds) applied to the source.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Trim {
    pub start: Option<f64>,
    pub end: Option<f64>,
}

impl Trim {
    pub fn is_none(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }
}

/// Build the full argument list. `input`/`output` must be absolute paths.
pub fn build_args(preset: &Preset, input: &Path, output: &Path, hw: Option<HwKind>) -> Vec<String> {
    build_args_trim(preset, input, output, hw, Trim::default())
}

/// Same as [`build_args`] with optional trimming. `-ss` before `-i` (fast + accurate when re-encoding),
/// `-t` as an output option so the length is exact.
pub fn build_args_trim(preset: &Preset, input: &Path, output: &Path, hw: Option<HwKind>, trim: Trim) -> Vec<String> {
    build_args_full(preset, input, output, hw, trim, &AudioChoice::default())
}

/// Full builder: trimming plus the per-job audio choice (stream, copy/convert/remove, downmix).
pub fn build_args_full(preset: &Preset, input: &Path, output: &Path, hw: Option<HwKind>, trim: Trim, audio: &AudioChoice) -> Vec<String> {
    build_args_pass(preset, input, output, hw, trim, audio, None)
}

/// Two-pass support: `pass` = (1 or 2, path prefix of the pass log file). Pass 1 analyses only
/// (no audio, null output — pass the platform's null device as `output`).
pub fn build_args_pass(preset: &Preset, input: &Path, output: &Path, hw: Option<HwKind>, trim: Trim, audio: &AudioChoice, pass: Option<(u8, &Path)>) -> Vec<String> {
    let mut a: Vec<String> = ["-hide_banner", "-nostdin", "-y", "-nostats", "-progress", "pipe:1", "-protocol_whitelist", "file"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    if let Some(st) = trim.start.filter(|v| v.is_finite() && *v > 0.0) {
        a.extend([s("-ss"), format!("{st:.3}")]);
    }
    a.push(s("-i"));
    a.push(input.to_string_lossy().into_owned());
    if let Some(end) = trim.end.filter(|v| v.is_finite() && *v > 0.0) {
        let len = end - trim.start.unwrap_or(0.0).max(0.0);
        if len > 0.0 {
            a.extend([s("-t"), format!("{len:.3}")]);
        }
    }
    a.extend([s("-sn"), s("-dn"), s("-map_metadata"), s("0")]);

    let first_pass = matches!(pass, Some((1, _)));
    let amap = format!("0:a:{}", audio.stream_index());
    if let Some(v) = &preset.video {
        a.extend([s("-map"), s("0:v:0")]);
        if !first_pass && audio.mode != AudioMode::None && (preset.audio.is_some() || audio.mode == AudioMode::Copy) {
            a.extend([s("-map"), format!("{amap}?")]);
        }
        let mut filters: Vec<String> = Vec::new();
        if let Some(h) = v.max_height {
            filters.push(format!("scale=-2:'min({h},ih)'"));
        }
        if let Some(f) = v.fps {
            filters.push(format!("fps={f}"));
        }
        if !filters.is_empty() {
            a.extend([s("-vf"), filters.join(",")]);
        }
        video_args(v, &preset.container, hw, &mut a);
        if first_pass {
            a.push(s("-an"));
        } else {
            match (&preset.audio, audio.mode) {
                (_, AudioMode::None) => a.push(s("-an")),
                (_, AudioMode::Copy) => a.extend([s("-c:a"), s("copy")]),
                (Some(au), AudioMode::Encode) => audio_args(au, audio.channels, &mut a),
                (None, AudioMode::Encode) => a.push(s("-an")),
            }
        }
        if let Some((n, log)) = pass {
            a.extend([s("-pass"), n.to_string(), s("-passlogfile"), log.to_string_lossy().into_owned()]);
        }
        if first_pass {
            a.extend([s("-f"), s("null")]);
        } else if preset.container == "mp4" || preset.container == "mov" {
            a.extend([s("-movflags"), s("+faststart")]);
        }
    } else {
        a.extend([s("-vn"), s("-map"), amap]);
        match (&preset.audio, audio.mode) {
            (_, AudioMode::Copy) => a.extend([s("-c:a"), s("copy")]),
            (Some(au), _) => audio_args(au, audio.channels, &mut a),
            (None, _) => {}
        }
    }
    a.push(output.to_string_lossy().into_owned());
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(id: &str) -> Preset {
        builtin_presets().into_iter().find(|p| p.id == id).unwrap()
    }
    fn args(id: &str, hw: Option<HwKind>) -> Vec<String> {
        build_args(&get(id), Path::new("C:\\in put.mov"), Path::new("C:\\out.tmp.mp4"), hw)
    }
    fn has_pair(a: &[String], k: &str, v: &str) -> bool {
        a.windows(2).any(|w| w[0] == k && w[1] == v)
    }

    #[test]
    fn h264_software_args() {
        let a = args("h264-1080", None);
        assert!(has_pair(&a, "-c:v", "libx264"));
        assert!(has_pair(&a, "-crf", "21"));
        assert!(has_pair(&a, "-vf", "scale=-2:'min(1080,ih)'"));
        assert!(has_pair(&a, "-c:a", "aac"));
        assert!(has_pair(&a, "-b:a", "192k"));
        assert!(has_pair(&a, "-movflags", "+faststart"));
        assert_eq!(a.last().unwrap(), "C:\\out.tmp.mp4");
    }

    #[test]
    fn input_with_spaces_stays_one_argument() {
        let a = args("h264-1080", None);
        assert!(a.iter().any(|x| x == "C:\\in put.mov"));
        assert!(has_pair(&a, "-protocol_whitelist", "file"));
    }

    #[test]
    fn hostile_filename_is_just_an_argument() {
        let p = get("h264-1080");
        let a = build_args(&p, Path::new("C:\\a; calc & -f.mov"), Path::new("C:\\o.mp4"), None);
        assert!(a.iter().any(|x| x == "C:\\a; calc & -f.mov"));
        let i = a.iter().position(|x| x == "-i").unwrap();
        assert_eq!(a[i + 1], "C:\\a; calc & -f.mov");
    }

    #[test]
    fn nvenc_uses_hw_encoder_with_cq() {
        let a = args("h264-1080", Some(HwKind::Nvenc));
        assert!(has_pair(&a, "-c:v", "h264_nvenc"));
        assert!(has_pair(&a, "-cq", "21"));
    }

    #[test]
    fn hw_ignored_when_codec_unsupported() {
        let a = args("vp9-1080-webm", Some(HwKind::Nvenc));
        assert!(has_pair(&a, "-c:v", "libvpx-vp9"));
        let a = args("prores422", Some(HwKind::Nvenc));
        assert!(has_pair(&a, "-c:v", "prores_ks"));
    }

    #[test]
    fn audio_only_drops_video() {
        let a = args("flac", None);
        assert!(a.contains(&"-vn".to_string()));
        assert!(has_pair(&a, "-c:a", "flac"));
        assert!(!a.iter().any(|x| x == "-c:v"));
    }

    #[test]
    fn trim_adds_seek_and_length() {
        let p = get("h264-1080");
        let a = build_args_trim(&p, Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim { start: Some(2.5), end: Some(7.0) });
        let ss = a.iter().position(|x| x == "-ss").unwrap();
        let i = a.iter().position(|x| x == "-i").unwrap();
        assert!(ss < i, "-ss must come before -i");
        assert_eq!(a[ss + 1], "2.500");
        assert!(has_pair(&a, "-t", "4.500"));
        let none = build_args_trim(&p, Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim::default());
        assert!(!none.contains(&"-ss".to_string()) && !none.contains(&"-t".to_string()));
        let bad = build_args_trim(&p, Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim { start: Some(9.0), end: Some(3.0) });
        assert!(!bad.contains(&"-t".to_string()), "end before start must not produce -t");
    }

    #[test]
    fn strip_metadata_replaces_the_copy_policy() {
        let mut a = build_args(&get("h264-1080"), Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None);
        assert!(has_pair(&a, "-map_metadata", "0"));
        strip_metadata(&mut a);
        assert!(has_pair(&a, "-map_metadata", "-1") && has_pair(&a, "-map_chapters", "-1"));
        assert!(!has_pair(&a, "-map_metadata", "0"));
        assert_eq!(a.last().unwrap(), "C:/o.mp4", "output must stay last");
    }

    fn full(id: &str, c: AudioChoice) -> Vec<String> {
        build_args_full(&get(id), Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim::default(), &c)
    }

    #[test]
    fn audio_stream_selection_maps_that_stream() {
        let a = full("h264-1080", AudioChoice { stream: Some(2), ..Default::default() });
        assert!(has_pair(&a, "-map", "0:a:2?"));
        let a = full("flac", AudioChoice { stream: Some(1), ..Default::default() });
        assert!(has_pair(&a, "-map", "0:a:1"));
    }

    #[test]
    fn audio_copy_none_and_downmix() {
        let a = full("h264-1080", AudioChoice { mode: AudioMode::Copy, ..Default::default() });
        assert!(has_pair(&a, "-c:a", "copy") && !a.contains(&"-b:a".to_string()));
        let a = full("h264-1080", AudioChoice { mode: AudioMode::None, ..Default::default() });
        assert!(a.contains(&"-an".to_string()) && !a.iter().any(|x| x.starts_with("0:a")));
        let a = full("h264-1080", AudioChoice { channels: Some(1), ..Default::default() });
        assert!(has_pair(&a, "-ac", "1") && has_pair(&a, "-c:a", "aac"));
    }

    #[test]
    fn default_audio_choice_matches_old_behaviour() {
        let old = build_args_trim(&get("h264-1080"), Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim::default());
        assert_eq!(old, full("h264-1080", AudioChoice::default()));
        assert!(has_pair(&old, "-map", "0:a:0?"));
    }

    fn with_video(id: &str, f: impl FnOnce(&mut VideoSpec)) -> Vec<String> {
        let mut p = get(id);
        f(p.video.as_mut().unwrap());
        p.validate().unwrap();
        build_args_full(&p, Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim::default(), &AudioChoice::default())
    }

    #[test]
    fn advanced_h264_settings_reach_ffmpeg() {
        let a = with_video("h264-1080", |v| {
            v.speed = Some(Speed::Slow);
            v.profile = Some("high".into());
            v.level = Some("4.1".into());
            v.keyint_secs = Some(2.0);
        });
        assert!(has_pair(&a, "-preset", "slow") && has_pair(&a, "-profile:v", "high") && has_pair(&a, "-level", "4.1"));
        assert!(has_pair(&a, "-force_key_frames", "expr:gte(t,n_forced*2)"));
    }

    #[test]
    fn ten_bit_changes_pixel_format_per_codec() {
        assert!(has_pair(&with_video("h264-1080", |v| v.bit_depth = Some(10)), "-pix_fmt", "yuv420p10le"));
        assert!(has_pair(&with_video("hevc-1080", |v| v.bit_depth = Some(10)), "-pix_fmt", "yuv420p10le"));
        let vp9 = with_video("vp9-1080-webm", |v| v.bit_depth = Some(10));
        assert!(has_pair(&vp9, "-profile:v", "2") && has_pair(&vp9, "-pix_fmt", "yuv420p10le"));
        assert!(has_pair(&with_video("h264-1080", |_| {}), "-pix_fmt", "yuv420p"));
    }

    #[test]
    fn speed_maps_per_encoder() {
        assert!(has_pair(&with_video("av1-1080-webm", |v| v.speed = Some(Speed::Fast)), "-preset", "10"));
        assert!(has_pair(&with_video("vp9-1080-webm", |v| v.speed = Some(Speed::Slow)), "-cpu-used", "1"));
        assert!(has_pair(&with_video("hevc-1080", |v| v.speed = Some(Speed::Fast)), "-preset", "fast"));
    }

    #[test]
    fn two_pass_arguments() {
        let mut p = get("h264-1080");
        p.video.as_mut().unwrap().rate = RateControl::Bitrate(3000);
        p.video.as_mut().unwrap().two_pass = true;
        let log = Path::new("C:/tmp/job-pass");
        let p1 = build_args_pass(&p, Path::new("C:/in.mov"), Path::new("NUL"), None, Trim::default(), &AudioChoice::default(), Some((1, log)));
        assert!(has_pair(&p1, "-pass", "1") && has_pair(&p1, "-passlogfile", "C:/tmp/job-pass") && has_pair(&p1, "-f", "null") && p1.contains(&"-an".to_string()));
        assert!(!p1.iter().any(|x| x.starts_with("0:a")) && !p1.contains(&"+faststart".to_string()));
        let p2 = build_args_pass(&p, Path::new("C:/in.mov"), Path::new("C:/o.mp4"), None, Trim::default(), &AudioChoice::default(), Some((2, log)));
        assert!(has_pair(&p2, "-pass", "2") && has_pair(&p2, "-c:a", "aac") && p2.contains(&"+faststart".to_string()));
    }

    #[test]
    fn prores_sets_profile_and_pixfmt() {
        let a = args("prores4444", None);
        assert!(has_pair(&a, "-profile:v", "4"));
        assert!(has_pair(&a, "-pix_fmt", "yuv444p10le"));
    }
}
