//! Post-export verification: compare what ffprobe sees in the output with what the preset promised.
//! Pure function over already-probed data so it is easy to test.

use crate::preset::{Preset, VideoCodec};
use cs_probe::MediaInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Verify {
    /// True when every check passed.
    pub ok: bool,
    /// Human-readable findings (empty when ok).
    pub notes: Vec<String>,
    pub size_bytes: Option<u64>,
    pub duration_secs: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

fn expected_codec(c: VideoCodec) -> &'static str {
    match c {
        VideoCodec::H264 => "h264",
        VideoCodec::Hevc => "hevc",
        VideoCodec::Av1 => "av1",
        VideoCodec::Vp9 => "vp9",
        VideoCodec::Prores422 | VideoCodec::Prores4444 => "prores",
        VideoCodec::Ffv1 => "ffv1",
    }
}

/// `expected_secs`: length the output should have (trimmed source length).
pub fn verify_output(out: &MediaInfo, preset: &Preset, expected_secs: Option<f64>, source_has_audio: bool) -> Verify {
    let mut notes = vec![];
    if out.size_bytes.unwrap_or(0) == 0 {
        notes.push("output file is empty".to_string());
    }
    match (&preset.video, &out.video) {
        (Some(v), Some(ov)) => {
            let want = expected_codec(v.codec);
            if ov.codec != want {
                notes.push(format!("video codec is {} but the preset asked for {want}", ov.codec));
            }
            if let Some(h) = v.max_height {
                if ov.height > h {
                    notes.push(format!("height {} is above the preset limit {h}", ov.height));
                }
            }
        }
        (Some(_), None) => notes.push("output has no video stream".into()),
        (None, Some(_)) => notes.push("audio-only preset produced a video stream".into()),
        (None, None) => {}
    }
    if preset.audio.is_some() && source_has_audio && out.audio.is_none() {
        notes.push("source had audio but the output has none".into());
    }
    if let (Some(exp), Some(got)) = (expected_secs, out.duration_secs) {
        let tol = (exp * 0.03).max(0.6);
        if (got - exp).abs() > tol {
            notes.push(format!("duration is {got:.1}s, expected about {exp:.1}s"));
        }
    } else if out.duration_secs.is_none() {
        notes.push("could not read the output duration".into());
    }
    Verify {
        ok: notes.is_empty(),
        notes,
        size_bytes: out.size_bytes,
        duration_secs: out.duration_secs,
        video_codec: out.video.as_ref().map(|v| v.codec.clone()),
        audio_codec: out.audio.as_ref().map(|a| a.codec.clone()),
        width: out.video.as_ref().map(|v| v.width),
        height: out.video.as_ref().map(|v| v.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::builtin_presets;
    use cs_probe::{AudioInfo, VideoInfo};

    fn info(vcodec: Option<&str>, h: u32, acodec: Option<&str>, dur: Option<f64>, size: u64) -> MediaInfo {
        MediaInfo {
            path: "o".into(),
            file_name: "o".into(),
            format_name: "mp4".into(),
            duration_secs: dur,
            size_bytes: Some(size),
            bit_rate: None,
            video: vcodec.map(|c| VideoInfo { codec: c.into(), width: h * 16 / 9, height: h, fps: Some(30.0), pix_fmt: None, field_order: None }),
            audio: acodec.map(|c| AudioInfo { codec: c.into(), channels: 2, sample_rate: Some(48000) }),
        }
    }
    fn preset(id: &str) -> Preset {
        builtin_presets().into_iter().find(|p| p.id == id).unwrap()
    }

    #[test]
    fn good_output_passes() {
        let v = verify_output(&info(Some("h264"), 1080, Some("aac"), Some(10.1), 5000), &preset("h264-1080"), Some(10.0), true);
        assert!(v.ok, "{:?}", v.notes);
        assert_eq!(v.video_codec.as_deref(), Some("h264"));
    }

    #[test]
    fn detects_wrong_codec_height_duration_and_missing_audio() {
        let v = verify_output(&info(Some("hevc"), 2160, None, Some(3.0), 5000), &preset("h264-1080"), Some(10.0), true);
        assert!(!v.ok);
        assert_eq!(v.notes.len(), 4, "{:?}", v.notes);
    }

    #[test]
    fn empty_file_and_missing_video_are_flagged() {
        let v = verify_output(&info(None, 0, Some("aac"), Some(10.0), 0), &preset("h264-1080"), Some(10.0), true);
        assert!(v.notes.iter().any(|n| n.contains("empty")));
        assert!(v.notes.iter().any(|n| n.contains("no video")));
    }

    #[test]
    fn audio_only_preset_rejects_video_and_tolerates_silent_source() {
        let v = verify_output(&info(None, 0, Some("aac"), Some(5.0), 100), &preset("aac-256"), Some(5.0), true);
        assert!(v.ok);
        let v = verify_output(&info(Some("h264"), 100, Some("aac"), Some(5.0), 100), &preset("aac-256"), Some(5.0), true);
        assert!(!v.ok);
        let v = verify_output(&info(Some("h264"), 720, None, Some(5.0), 100), &preset("h264-720"), Some(5.0), false);
        assert!(v.ok, "no audio in source -> none expected: {:?}", v.notes);
    }
}
