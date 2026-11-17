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

/// Encoder effort: faster = bigger files, slower = smaller files at the same quality.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Speed {
    Fast,
    Medium,
    Slow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoSpec {
    pub codec: VideoCodec,
    pub rate: RateControl,
    /// Scale down so the height is at most this value (never upscales).
    pub max_height: Option<u32>,
    pub fps: Option<f64>,
    /// 8 (default) or 10. 10-bit needs a software encoder and a capable player.
    #[serde(default)]
    pub bit_depth: Option<u32>,
    /// Force a keyframe every N seconds (streaming / seeking). Software encoders only: hardware encoders ignore it.
    #[serde(default)]
    pub keyint_secs: Option<f64>,
    #[serde(default)]
    pub speed: Option<Speed>,
    /// Two passes give a more accurate bitrate. Bitrate mode with H.264, HEVC or VP9 only.
    #[serde(default)]
    pub two_pass: bool,
    /// H.264 only: baseline, main or high.
    #[serde(default)]
    pub profile: Option<String>,
    /// H.264 only, e.g. "4.1".
    #[serde(default)]
    pub level: Option<String>,
}

impl VideoSpec {
    /// True when the user asked for something only the software encoder can do exactly.
    pub fn needs_software(&self) -> bool {
        self.bit_depth == Some(10) || self.two_pass || self.profile.is_some() || self.level.is_some() || self.speed.is_some() || self.keyint_secs.is_some()
    }
}

pub const H264_LEVELS: [&str; 12] = ["3.0", "3.1", "3.2", "4.0", "4.1", "4.2", "5.0", "5.1", "5.2", "6.0", "6.1", "6.2"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSpec {
    pub codec: AudioCodec,
    pub bitrate_kbps: Option<u32>,
    pub channels: Option<u32>,
}

/// Current preset file format. v1 had no `schema` field; v2 adds `schema` and `description`.
pub const PRESET_SCHEMA: u32 = 2;

fn default_schema() -> u32 {
    PRESET_SCHEMA
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    /// Format version of this preset (see [`PRESET_SCHEMA`]).
    #[serde(default = "default_schema")]
    pub schema: u32,
    #[serde(default)]
    pub description: Option<String>,
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
            let lossy = matches!(v.codec, VideoCodec::H264 | VideoCodec::Hevc | VideoCodec::Av1 | VideoCodec::Vp9);
            if let Some(b) = v.bit_depth {
                if b != 8 && b != 10 {
                    return err("bit depth must be 8 or 10");
                }
                if b == 10 && !lossy {
                    return err(format!("{} has a fixed bit depth", v.codec.name()));
                }
            }
            if let Some(k) = v.keyint_secs {
                if !k.is_finite() || !(0.5..=20.0).contains(&k) {
                    return err("keyframe interval must be 0.5-20 seconds");
                }
            }
            if v.speed.is_some() && !lossy {
                return err(format!("{} has no speed setting", v.codec.name()));
            }
            if v.two_pass {
                if !matches!(v.rate, RateControl::Bitrate(_)) {
                    return err("two-pass needs a target bitrate (not quality mode)");
                }
                if !matches!(v.codec, VideoCodec::H264 | VideoCodec::Hevc | VideoCodec::Vp9) {
                    return err(format!("two-pass is not supported for {}", v.codec.name()));
                }
            }
            if let Some(pr) = &v.profile {
                if v.codec != VideoCodec::H264 {
                    return err("a profile can only be set for H.264");
                }
                if !["baseline", "main", "high"].contains(&pr.as_str()) {
                    return err("H.264 profile must be baseline, main or high");
                }
                if v.bit_depth == Some(10) {
                    return err("an H.264 profile cannot be forced together with 10-bit");
                }
            }
            if let Some(l) = &v.level {
                if v.codec != VideoCodec::H264 {
                    return err("a level can only be set for H.264");
                }
                if !H264_LEVELS.contains(&l.as_str()) {
                    return err("H.264 level must be one of 3.0 … 6.2");
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

    /// Parse a preset file of any supported version, migrate it to the current one and validate it.
    pub fn from_json(s: &str) -> Result<Preset, PresetError> {
        let v: serde_json::Value = serde_json::from_str(s).map_err(|e| PresetError(e.to_string()))?;
        Preset::from_value(v)
    }

    pub fn from_value(v: serde_json::Value) -> Result<Preset, PresetError> {
        let version = match v.get("schema") {
            None | Some(serde_json::Value::Null) => 1,
            Some(n) => n.as_u64().ok_or_else(|| PresetError("preset 'schema' must be a number".into()))? as u32,
        };
        if version == 0 {
            return err("preset 'schema' must be 1 or higher");
        }
        if version > PRESET_SCHEMA {
            return err(format!("this preset was made by a newer Squeeze (format {version}); update Squeeze to use it"));
        }
        let mut p: Preset = serde_json::from_value(v).map_err(|e| PresetError(e.to_string()))?;
        // v1 -> v2 only added optional fields, so migrating means stamping the current version
        p.schema = PRESET_SCHEMA;
        if p.description.as_ref().is_some_and(|d| d.chars().count() > 300) {
            return err("preset description is too long (max 300 characters)");
        }
        p.validate()?;
        Ok(p)
    }
}

fn v(codec: VideoCodec, rate: RateControl, h: Option<u32>) -> Option<VideoSpec> {
    Some(VideoSpec { codec, rate, max_height: h, fps: None, bit_depth: None, keyint_secs: None, speed: None, two_pass: false, profile: None, level: None })
}
fn a(codec: AudioCodec, kbps: Option<u32>) -> Option<AudioSpec> {
    Some(AudioSpec { codec, bitrate_kbps: kbps, channels: None })
}
fn p(id: &str, name: &str, group: &str, container: &str, video: Option<VideoSpec>, audio: Option<AudioSpec>) -> Preset {
    Preset { schema: PRESET_SCHEMA, description: None, id: id.into(), name: name.into(), group: group.into(), container: container.into(), video, audio }
}

pub fn builtin_presets() -> Vec<Preset> {
    use AudioCodec as A;
    use RateControl::*;
    use VideoCodec as V;
    vec![
        p("h264-1080", "H.264 MP4 — 1080p", "Web", "mp4", v(V::H264, Crf(21), Some(1080)), a(A::Aac, Some(192))),
        p("h264-720", "H.264 MP4 — 720p", "Web", "mp4", v(V::H264, Crf(22), Some(720)), a(A::Aac, Some(160))),
        p("hevc-1080", "HEVC MP4 — 1080p", "Web", "mp4", v(V::Hevc, Crf(24), Some(1080)), a(A::Aac, Some(192))),
        p("h264-1080-small", "H.264 MP4 — 1080p (smaller file)", "Web", "mp4", v(V::H264, Crf(26), Some(1080)), a(A::Aac, Some(128))),
        p("hevc-1080-small", "HEVC MP4 — 1080p (smaller file)", "Web", "mp4", v(V::Hevc, Crf(28), Some(1080)), a(A::Aac, Some(128))),
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
    fn v1_preset_without_schema_is_migrated() {
        let v1 = r#"{"id":"old-one","name":"Old one","group":"Custom","container":"mp4",
          "video":{"codec":"h264","rate":{"mode":"crf","value":23},"max_height":720,"fps":null},
          "audio":{"codec":"aac","bitrate_kbps":128,"channels":null}}"#;
        let p = Preset::from_json(v1).unwrap();
        assert_eq!(p.schema, PRESET_SCHEMA);
        assert_eq!(p.description, None);
        assert_eq!(p.video.unwrap().max_height, Some(720));
    }

    #[test]
    fn newer_or_invalid_schema_is_refused_with_a_reason() {
        let future = r#"{"schema":99,"id":"x","name":"X","group":"Custom","container":"mp4","video":null,"audio":{"codec":"aac","bitrate_kbps":128,"channels":null}}"#;
        let e = Preset::from_json(future).unwrap_err().to_string();
        assert!(e.contains("newer Squeeze"), "{e}");
        assert!(Preset::from_json(r#"{"schema":"two"}"#).is_err());
        assert!(Preset::from_json(r#"{"schema":0,"id":"x"}"#).is_err());
    }

    #[test]
    fn all_builtins_use_the_current_schema_and_export_it() {
        for p in builtin_presets() {
            assert_eq!(p.schema, PRESET_SCHEMA);
            assert!(serde_json::to_string(&p).unwrap().contains("\"schema\":2"));
        }
    }

    fn adv(f: impl FnOnce(&mut VideoSpec)) -> Preset {
        let mut p = builtin_presets().into_iter().find(|p| p.id == "h264-1080").unwrap();
        f(p.video.as_mut().unwrap());
        p
    }

    #[test]
    fn advanced_settings_are_validated() {
        assert!(adv(|v| v.bit_depth = Some(10)).validate().is_ok());
        assert!(adv(|v| v.bit_depth = Some(12)).validate().is_err());
        assert!(adv(|v| v.keyint_secs = Some(2.0)).validate().is_ok());
        assert!(adv(|v| v.keyint_secs = Some(0.0)).validate().is_err());
        assert!(adv(|v| v.keyint_secs = Some(f64::NAN)).validate().is_err());
        assert!(adv(|v| v.speed = Some(Speed::Slow)).validate().is_ok());
        assert!(adv(|v| v.two_pass = true).validate().is_err(), "two-pass needs bitrate mode");
        assert!(adv(|v| { v.two_pass = true; v.rate = RateControl::Bitrate(3000); }).validate().is_ok());
        assert!(adv(|v| { v.two_pass = true; v.rate = RateControl::Bitrate(3000); v.codec = VideoCodec::Av1; }).validate().is_err());
        assert!(adv(|v| v.profile = Some("high".into())).validate().is_ok());
        assert!(adv(|v| v.profile = Some("ultra".into())).validate().is_err());
        assert!(adv(|v| { v.profile = Some("high".into()); v.bit_depth = Some(10); }).validate().is_err());
        assert!(adv(|v| v.level = Some("4.1".into())).validate().is_ok());
        assert!(adv(|v| v.level = Some("9.9".into())).validate().is_err());
        let mut hevc = builtin_presets().into_iter().find(|p| p.id == "hevc-1080").unwrap();
        hevc.video.as_mut().unwrap().profile = Some("main".into());
        assert!(hevc.validate().is_err(), "profile is H.264 only");
        let mut pr = builtin_presets().into_iter().find(|p| p.id == "prores422").unwrap();
        pr.video.as_mut().unwrap().bit_depth = Some(10);
        assert!(pr.validate().is_err());
    }

    #[test]
    fn old_presets_without_advanced_fields_still_load() {
        let p = Preset::from_json(r#"{"id":"o","name":"O","group":"Custom","container":"mp4","video":{"codec":"h264","rate":{"mode":"crf","value":22},"max_height":null,"fps":null},"audio":{"codec":"aac","bitrate_kbps":128,"channels":null}}"#).unwrap();
        let v = p.video.unwrap();
        assert!(!v.needs_software() && v.bit_depth.is_none() && !v.two_pass);
    }

    #[test]
    fn rejects_garbage_json() {
        assert!(Preset::from_json("{}").is_err());
        assert!(Preset::from_json("nope").is_err());
    }
}
