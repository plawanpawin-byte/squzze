//! Encoding presets: serde schema, validation and the built-in list.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    Hevc,
    Av1,
    Vp9,
    Prores422,
    Prores4444,
    Ffv1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    Aac,
    Opus,
    Flac,
    Pcm24,
    Mp3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode", content = "value", rename_all = "snake_case")]
pub enum RateControl {
    /// Constant quality (CRF). Lower = better; range depends on codec.
    Crf(u32),
    /// Target bitrate in kbit/s.
    Bitrate(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoSpec {
    pub codec: VideoCodec,
    pub rate: RateControl,
    /// Scale down so the height is at most this value (never upscales).
    pub max_height: Option<u32>,
    pub fps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSpec {
    pub codec: AudioCodec,
    pub bitrate_kbps: Option<u32>,
    pub channels: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub group: String,
    /// File extension / container: mp4, mov, mkv, webm, m4a, opus, flac, wav, mp3.
    pub container: String,
    pub video: Option<VideoSpec>,
    pub audio: Option<AudioSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct PresetError(pub String);

fn err<T>(s: impl Into<String>) -> Result<T, PresetError> {
    Err(PresetError(s.into()))
}

impl VideoCodec {
    pub fn crf_range(self) -> (u32, u32) {
        match self {
            VideoCodec::H264 | VideoCodec::Hevc => (0, 51),
            VideoCodec::Av1 | VideoCodec::Vp9 => (0, 63),
            _ => (0, 0),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            VideoCodec::H264 => "H.264",
            VideoCodec::Hevc => "HEVC",
            VideoCodec::Av1 => "AV1",
            VideoCodec::Vp9 => "VP9",
            VideoCodec::Prores422 => "ProRes 422",
            VideoCodec::Prores4444 => "ProRes 4444",
            VideoCodec::Ffv1 => "FFV1",
        }
    }
}

fn video_ok(container: &str, c: VideoCodec) -> bool {
    use VideoCodec::*;
    match container {
        "mp4" => matches!(c, H264 | Hevc | Av1),
        "mov" => matches!(c, H264 | Hevc | Prores422 | Prores4444),
        "mkv" => true,
        "webm" => matches!(c, Vp9 | Av1),
        _ => false,
    }
}

fn audio_ok(container: &str, c: AudioCodec) -> bool {
    use AudioCodec::*;
    match container {
        "mp4" => matches!(c, Aac | Mp3 | Opus),
        "mov" => matches!(c, Aac | Pcm24),
        "mkv" => true,
        "webm" => matches!(c, Opus),
        "m4a" => matches!(c, Aac),
        "opus" => matches!(c, Opus),
        "flac" => matches!(c, Flac),
        "wav" => matches!(c, Pcm24),
        "mp3" => matches!(c, Mp3),
        _ => false,
    }
}

impl Preset {
    pub fn is_audio_only(&self) -> bool {
        self.video.is_none()
    }

    pub fn validate(&self) -> Result<(), PresetError> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return err("preset id must be 1-64 chars of a-z, 0-9, '-' or '_'");
        }
        if self.name.trim().is_empty() || self.name.chars().count() > 80 {
            return err("preset name must be 1-80 characters");
        }
        if self.group.chars().count() > 40 {
            return err("preset group too long");
        }
        let video_containers = ["mp4", "mov", "mkv", "webm"];
        let audio_containers = ["m4a", "opus", "flac", "wav", "mp3", "mkv", "mp4", "mov", "webm"];
        match (&self.video, &self.audio) {
            (None, None) => return err("preset needs video or audio"),
            (None, Some(_)) if !audio_containers[..5].contains(&self.container.as_str()) => {
                return err(format!("container '{}' is not an audio-only format", self.container))
            }
            (Some(_), _) if !video_containers.contains(&self.container.as_str()) => {
                return err(format!("unsupported video container '{}'", self.container))
            }
            _ => {}
        }
        if let Some(v) = &self.video {
            if !video_ok(&self.container, v.codec) {
                return err(format!("{} cannot be stored in .{}", v.codec.name(), self.container));
            }
            match v.rate {
                RateControl::Crf(q) => {
                    let (lo, hi) = v.codec.crf_range();
                    if hi == 0 && q != 0 {
                        return err(format!("{} does not use quality (CRF)", v.codec.name()));
                    }
                    if q < lo || q > hi {
                        return err(format!("quality must be {lo}-{hi} for {}", v.codec.name()));
                    }
                }
                RateControl::Bitrate(k) => {
                    if !(50..=500_000).contains(&k) {
                        return err("video bitrate must be 50-500000 kbit/s");
                    }
                    if matches!(v.codec, VideoCodec::Prores422 | VideoCodec::Prores4444 | VideoCodec::Ffv1) {
                        return err(format!("{} does not support a target bitrate", v.codec.name()));
                    }
                }
            }
            if let Some(h) = v.max_height {
                if !(144..=8640).contains(&h) {
                    return err("max height must be 144-8640");
                }
            }
            if let Some(f) = v.fps {
                if !(1.0..=240.0).contains(&f) || !f.is_finite() {
                    return err("fps must be 1-240");
                }
            }
        }
        if let Some(a) = &self.audio {
            if !audio_ok(&self.container, a.codec) {
                return err(format!("{:?} audio cannot be stored in .{}", a.codec, self.container));
            }
            if let Some(b) = a.bitrate_kbps {
                if !(16..=1536).contains(&b) {
                    return err("audio bitrate must be 16-1536 kbit/s");
                }
            }
            if let Some(c) = a.channels {
                if !(1..=8).contains(&c) {
                    return err("audio channels must be 1-8");
                }
            }
        }
        Ok(())
    }

    pub fn from_json(s: &str) -> Result<Preset, PresetError> {
        let p: Preset = serde_json::from_str(s).map_err(|e| PresetError(e.to_string()))?;
        p.validate()?;
        Ok(p)
    }
}

fn v(codec: VideoCodec, rate: RateControl, h: Option<u32>) -> Option<VideoSpec> {
    Some(VideoSpec { codec, rate, max_height: h, fps: None })
}
fn a(codec: AudioCodec, kbps: Option<u32>) -> Option<AudioSpec> {
    Some(AudioSpec { codec, bitrate_kbps: kbps, channels: None })
}
fn p(id: &str, name: &str, group: &str, container: &str, video: Option<VideoSpec>, audio: Option<AudioSpec>) -> Preset {
    Preset { id: id.into(), name: name.into(), group: group.into(), container: container.into(), video, audio }
}

pub fn builtin_presets() -> Vec<Preset> {
    use AudioCodec as A;
    use RateControl::*;
    use VideoCodec as V;
    vec![
        p("h264-1080", "H.264 MP4 — 1080p", "Web", "mp4", v(V::H264, Crf(21), Some(1080)), a(A::Aac, Some(192))),
        p("h264-720", "H.264 MP4 — 720p", "Web", "mp4", v(V::H264, Crf(22), Some(720)), a(A::Aac, Some(160))),
        p("hevc-1080", "HEVC MP4 — 1080p", "Web", "mp4", v(V::Hevc, Crf(24), Some(1080)), a(A::Aac, Some(192))),
        p("av1-1080-webm", "AV1 WebM — 1080p", "Web", "webm", v(V::Av1, Crf(34), Some(1080)), a(A::Opus, Some(128))),
        p("vp9-1080-webm", "VP9 WebM — 1080p", "Web", "webm", v(V::Vp9, Crf(32), Some(1080)), a(A::Opus, Some(128))),
        p("social-720", "Social share 720p (small)", "Social", "mp4", v(V::H264, Crf(26), Some(720)), a(A::Aac, Some(128))),
        p("prores422", "ProRes 422 MOV", "Archive", "mov", v(V::Prores422, Crf(0), None), a(A::Pcm24, None)),
        p("prores4444", "ProRes 4444 MOV", "Archive", "mov", v(V::Prores4444, Crf(0), None), a(A::Pcm24, None)),
        p("ffv1-mkv", "FFV1 MKV (lossless)", "Archive", "mkv", v(V::Ffv1, Crf(0), None), a(A::Flac, None)),
        p("aac-256", "AAC 256 kbps (.m4a)", "Audio", "m4a", None, a(A::Aac, Some(256))),
        p("opus-128", "Opus 128 kbps (.opus)", "Audio", "opus", None, a(A::Opus, Some(128))),
        p("mp3-192", "MP3 192 kbps", "Audio", "mp3", None, a(A::Mp3, Some(192))),
        p("flac", "FLAC (lossless)", "Audio", "flac", None, a(A::Flac, None)),
        p("wav24", "WAV 24-bit", "Audio", "wav", None, a(A::Pcm24, None)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtins_validate_and_ids_unique() {
        let all = builtin_presets();
        let mut ids: Vec<_> = all.iter().map(|p| p.id.clone()).collect();
        for p in &all {
            p.validate().unwrap_or_else(|e| panic!("{}: {e}", p.id));
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
    }

    #[test]
    fn json_round_trip_is_identical() {
        for p in builtin_presets() {
            let s = serde_json::to_string(&p).unwrap();
            assert_eq!(Preset::from_json(&s).unwrap(), p);
        }
    }

    #[test]
    fn rejects_incompatible_and_out_of_range() {
        let mut p = builtin_presets().remove(0);
        p.container = "webm".into();
        assert!(p.validate().is_err()); // h264 in webm
        let mut p = builtin_presets().remove(0);
        p.video.as_mut().unwrap().rate = RateControl::Crf(99);
        assert!(p.validate().is_err());
        let mut p = builtin_presets().remove(0);
        p.id = "../evil".into();
        assert!(p.validate().is_err());
        let mut p = builtin_presets().remove(0);
        p.video.as_mut().unwrap().fps = Some(f64::NAN);
        assert!(p.validate().is_err());
    }

    #[test]
    fn rejects_garbage_json() {
        assert!(Preset::from_json("{}").is_err());
        assert!(Preset::from_json("nope").is_err());
    }
}
