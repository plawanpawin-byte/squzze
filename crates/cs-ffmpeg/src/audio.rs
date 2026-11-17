//! Per-job audio choices: which stream, copy vs re-encode, remove, downmix; validated before a job starts.

use crate::preset::Preset;
use cs_probe::MediaInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioMode {
    /// Encode with the preset's audio settings (default).
    #[default]
    Encode,
    /// Copy the original audio untouched (no quality loss, very fast).
    Copy,
    /// Drop the audio.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AudioChoice {
    /// Which audio stream of the source (0 = first). `None` = first.
    #[serde(default)]
    pub stream: Option<u32>,
    #[serde(default)]
    pub mode: AudioMode,
    /// Downmix to this many channels (Encode only). Never upmixes.
    #[serde(default)]
    pub channels: Option<u32>,
}

impl AudioChoice {
    pub fn is_default(&self) -> bool {
        *self == AudioChoice::default()
    }
    pub fn stream_index(&self) -> u32 {
        self.stream.unwrap_or(0)
    }
}

/// Can `codec` (an FFmpeg decoder name such as "aac" or "pcm_s16le") be stored unchanged in `container`?
pub fn copy_ok(container: &str, codec: &str) -> bool {
    let pcm = codec.starts_with("pcm_");
    match container {
        "mkv" => true,
        "mp4" => matches!(codec, "aac" | "mp3" | "opus" | "ac3" | "eac3" | "flac"),
        "mov" => matches!(codec, "aac" | "mp3" | "ac3") || pcm,
        "webm" => matches!(codec, "opus" | "vorbis"),
        "m4a" => matches!(codec, "aac"),
        "opus" => codec == "opus",
        "flac" => codec == "flac",
        "wav" => pcm,
        "mp3" => codec == "mp3",
        _ => false,
    }
}

pub fn validate(preset: &Preset, info: &MediaInfo, c: &AudioChoice) -> Result<(), String> {
    if c.is_default() {
        return Ok(());
    }
    let n = info.audio_streams.len() as u32;
    if c.mode == AudioMode::None {
        if preset.video.is_none() {
            return Err("An audio-only preset needs audio; \"Remove audio\" would leave an empty file.".into());
        }
        if c.channels.is_some() {
            return Err("Channels cannot be changed when the audio is removed.".into());
        }
        return Ok(());
    }
    if n == 0 {
        if c.stream.is_some() || c.mode == AudioMode::Copy {
            return Err("This file has no audio to select or copy.".into());
        }
        return Ok(());
    }
    let idx = c.stream_index();
    let Some(stream) = info.audio_streams.get(idx as usize) else {
        return Err(format!("Audio stream {} does not exist (the file has {n}).", idx + 1));
    };
    if c.mode == AudioMode::Copy {
        if c.channels.is_some() {
            return Err("Copying the original audio cannot change the number of channels. Choose \"Convert\" instead.".into());
        }
        if !copy_ok(&preset.container, &stream.codec) {
            return Err(format!(
                "The original audio ({}) cannot be stored unchanged in .{}. Choose \"Convert\" instead.",
                stream.codec, preset.container
            ));
        }
    }
    if let Some(ch) = c.channels {
        if !(1..=8).contains(&ch) {
            return Err("Channels must be between 1 and 8.".into());
        }
        if stream.channels > 0 && ch > stream.channels {
            return Err(format!("The source has {} channel(s); Squeeze only downmixes, it does not invent channels.", stream.channels));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::builtin_presets;
    use cs_probe::AudioInfo;

    fn info(streams: &[(&str, u32)]) -> MediaInfo {
        let audio_streams: Vec<AudioInfo> = streams.iter().enumerate().map(|(i, (c, ch))| AudioInfo { codec: (*c).into(), channels: *ch, index: i as u32, ..Default::default() }).collect();
        MediaInfo { path: "x".into(), file_name: "x".into(), format_name: "mov".into(), duration_secs: Some(1.0), size_bytes: Some(1), bit_rate: None, video: None, audio: audio_streams.first().cloned(), audio_streams, subtitles: vec![] }
    }
    fn p(id: &str) -> Preset {
        builtin_presets().into_iter().find(|x| x.id == id).unwrap()
    }

    #[test]
    fn default_choice_always_valid() {
        assert!(validate(&p("h264-1080"), &info(&[]), &AudioChoice::default()).is_ok());
    }

    #[test]
    fn stream_must_exist() {
        let c = AudioChoice { stream: Some(2), ..Default::default() };
        assert!(validate(&p("h264-1080"), &info(&[("aac", 2), ("aac", 2)]), &c).is_err());
        let c = AudioChoice { stream: Some(1), ..Default::default() };
        assert!(validate(&p("h264-1080"), &info(&[("aac", 2), ("aac", 2)]), &c).is_ok());
    }

    #[test]
    fn copy_checks_container_compatibility() {
        let c = AudioChoice { mode: AudioMode::Copy, ..Default::default() };
        assert!(validate(&p("h264-1080"), &info(&[("aac", 2)]), &c).is_ok()); // mp4 + aac
        assert!(validate(&p("h264-1080"), &info(&[("pcm_s16le", 2)]), &c).is_err()); // mp4 + pcm
        assert!(validate(&p("vp9-1080-webm"), &info(&[("aac", 2)]), &c).is_err()); // webm + aac
        assert!(validate(&p("ffv1-mkv"), &info(&[("ac3", 6)]), &c).is_ok()); // mkv takes anything
        assert!(validate(&p("h264-1080"), &info(&[]), &c).is_err()); // nothing to copy
    }

    #[test]
    fn channels_only_downmix_and_not_with_copy() {
        let src = info(&[("aac", 6)]);
        assert!(validate(&p("h264-1080"), &src, &AudioChoice { channels: Some(2), ..Default::default() }).is_ok());
        assert!(validate(&p("h264-1080"), &info(&[("aac", 2)]), &AudioChoice { channels: Some(6), ..Default::default() }).is_err());
        assert!(validate(&p("h264-1080"), &src, &AudioChoice { channels: Some(0), ..Default::default() }).is_err());
        assert!(validate(&p("h264-1080"), &src, &AudioChoice { mode: AudioMode::Copy, channels: Some(2), ..Default::default() }).is_err());
    }

    #[test]
    fn removing_audio_needs_a_video_preset() {
        let c = AudioChoice { mode: AudioMode::None, ..Default::default() };
        assert!(validate(&p("h264-1080"), &info(&[("aac", 2)]), &c).is_ok());
        assert!(validate(&p("aac-256"), &info(&[("aac", 2)]), &c).is_err());
    }
}
