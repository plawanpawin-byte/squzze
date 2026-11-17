//! Turn a validated preset into an FFmpeg argument vector (never a shell string).

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
            out.extend([s("-c:v"), s("libx264"), s("-preset"), s("medium")]);
            rate_args(v.rate, out);
        }
        (None, Hevc) => {
            out.extend([s("-c:v"), s("libx265"), s("-preset"), s("medium"), s("-x265-params"), s("log-level=error")]);
            rate_args(v.rate, out);
            if container == "mp4" || container == "mov" {
                out.extend([s("-tag:v"), s("hvc1")]);
            }
        }
        (None, Av1) => {
            out.extend([s("-c:v"), s("libsvtav1"), s("-preset"), s("8")]);
            rate_args(v.rate, out);
        }
        (None, Vp9) => {
            out.extend([s("-c:v"), s("libvpx-vp9"), s("-row-mt"), s("1"), s("-deadline"), s("good"), s("-cpu-used"), s("2")]);
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
        out.extend([s("-pix_fmt"), s("yuv420p")]);
    }
}

fn rate_args(r: RateControl, out: &mut Vec<String>) {
    match r {
        RateControl::Crf(q) => out.extend([s("-crf"), q.to_string()]),
        RateControl::Bitrate(k) => out.extend([s("-b:v"), format!("{k}k")]),
    }
}

fn audio_args(a: &AudioSpec, out: &mut Vec<String>) {
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
    if let Some(c) = a.channels {
        out.extend([s("-ac"), c.to_string()]);
    }
}

/// Build the full argument list. `input`/`output` must be absolute paths.
pub fn build_args(preset: &Preset, input: &Path, output: &Path, hw: Option<HwKind>) -> Vec<String> {
    let mut a: Vec<String> = ["-hide_banner", "-nostdin", "-y", "-nostats", "-progress", "pipe:1", "-protocol_whitelist", "file", "-i"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    a.push(input.to_string_lossy().into_owned());
    a.extend([s("-sn"), s("-dn"), s("-map_metadata"), s("0")]);

    if let Some(v) = &preset.video {
        a.extend([s("-map"), s("0:v:0")]);
        if preset.audio.is_some() {
            a.extend([s("-map"), s("0:a:0?")]);
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
        if let Some(au) = &preset.audio {
            audio_args(au, &mut a);
        } else {
            a.push(s("-an"));
        }
        if preset.container == "mp4" || preset.container == "mov" {
            a.extend([s("-movflags"), s("+faststart")]);
        }
    } else {
        a.extend([s("-vn"), s("-map"), s("0:a:0")]);
        if let Some(au) = &preset.audio {
            audio_args(au, &mut a);
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
    fn prores_sets_profile_and_pixfmt() {
        let a = args("prores4444", None);
        assert!(has_pair(&a, "-profile:v", "4"));
        assert!(has_pair(&a, "-pix_fmt", "yuv444p10le"));
    }
}
