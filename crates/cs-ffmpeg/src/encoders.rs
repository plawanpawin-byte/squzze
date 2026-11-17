//! Which encoders the installed FFmpeg really has, and which presets can therefore run.

use crate::args::HwKind;
use crate::preset::{AudioCodec, Preset, VideoCodec};
use cs_probe::no_window;
use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Stdio};

/// Parse the output of `ffmpeg -encoders` into a set of encoder names.
pub fn parse_encoders(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut in_list = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("------") {
            in_list = true;
            continue;
        }
        if !in_list {
            continue;
        }
        let mut parts = t.split_whitespace();
        if let (Some(flags), Some(name)) = (parts.next(), parts.next()) {
            if flags.len() == 6 && matches!(flags.chars().next(), Some('V' | 'A' | 'S')) {
                out.insert(name.to_string());
            }
        }
    }
    out
}

/// Encoders reported by the installed FFmpeg (empty if it cannot be run).
pub fn list_encoders(ffmpeg: &Path) -> HashSet<String> {
    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-hide_banner", "-encoders"]).stdin(Stdio::null()).stderr(Stdio::null());
    no_window(&mut cmd);
    match cmd.output() {
        Ok(o) if o.status.success() => parse_encoders(&String::from_utf8_lossy(&o.stdout)),
        _ => HashSet::new(),
    }
}

pub fn software_video_encoder(c: VideoCodec) -> &'static str {
    match c {
        VideoCodec::H264 => "libx264",
        VideoCodec::Hevc => "libx265",
        VideoCodec::Av1 => "libsvtav1",
        VideoCodec::Vp9 => "libvpx-vp9",
        VideoCodec::Prores422 | VideoCodec::Prores4444 => "prores_ks",
        VideoCodec::Ffv1 => "ffv1",
    }
}

pub fn audio_encoder(c: AudioCodec) -> &'static str {
    match c {
        AudioCodec::Aac => "aac",
        AudioCodec::Opus => "libopus",
        AudioCodec::Mp3 => "libmp3lame",
        AudioCodec::Flac => "flac",
        AudioCodec::Pcm24 => "pcm_s24le",
    }
}

/// `None` when the preset can run; otherwise a message for the user.
/// A video codec is fine if the software encoder exists **or** a detected GPU can encode it.
pub fn unavailable_reason(p: &Preset, installed: &HashSet<String>, hw: &[HwKind]) -> Option<String> {
    if installed.is_empty() {
        return None; // FFmpeg missing is reported elsewhere; do not mark everything unavailable
    }
    if let Some(v) = &p.video {
        let sw = software_video_encoder(v.codec);
        let hw_ok = hw.iter().any(|h| h.encoder(v.codec).map(|e| installed.contains(e)).unwrap_or(false));
        if !installed.contains(sw) && !hw_ok {
            return Some(format!(
                "This FFmpeg has no '{sw}' encoder. Install a full build (winget install Gyan.FFmpeg) to use \"{}\".",
                p.name
            ));
        }
    }
    if let Some(a) = &p.audio {
        let enc = audio_encoder(a.codec);
        if !installed.contains(enc) {
            return Some(format!("This FFmpeg has no '{enc}' encoder, so \"{}\" is not available.", p.name));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::builtin_presets;

    const SAMPLE: &str = " Encoders:\n V..... = Video\n ------\n V....D libx264              libx264 H.264\n V..... libsvtav1            SVT-AV1\n A....D aac                  AAC\n S..... ssa                  ASS subtitles\n";

    #[test]
    fn parses_names_and_ignores_the_legend() {
        let e = parse_encoders(SAMPLE);
        assert!(e.contains("libx264") && e.contains("libsvtav1") && e.contains("aac") && e.contains("ssa"));
        assert!(!e.contains("=") && !e.contains("Video"));
    }

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }
    fn get(id: &str) -> Preset {
        builtin_presets().into_iter().find(|p| p.id == id).unwrap()
    }

    #[test]
    fn missing_software_encoder_makes_preset_unavailable() {
        let enc = set(&["libx264", "aac", "libopus"]);
        assert!(unavailable_reason(&get("h264-1080"), &enc, &[]).is_none());
        let why = unavailable_reason(&get("av1-1080-webm"), &enc, &[]).unwrap();
        assert!(why.contains("libsvtav1"), "{why}");
    }

    #[test]
    fn working_gpu_encoder_rescues_a_missing_software_one() {
        let enc = set(&["av1_nvenc", "libopus"]);
        assert!(unavailable_reason(&get("av1-1080-webm"), &enc, &[HwKind::Nvenc]).is_none());
        assert!(unavailable_reason(&get("av1-1080-webm"), &enc, &[]).is_some());
    }

    #[test]
    fn missing_audio_encoder_is_reported() {
        let enc = set(&["libx264"]);
        assert!(unavailable_reason(&get("h264-1080"), &enc, &[]).unwrap().contains("aac"));
    }

    #[test]
    fn unknown_ffmpeg_does_not_hide_every_preset() {
        assert!(unavailable_reason(&get("h264-1080"), &HashSet::new(), &[]).is_none());
    }
}
