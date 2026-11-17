//! Presets, FFmpeg argument building, process running and hardware detection.

pub mod args;
pub mod outpath;
pub mod preset;
pub mod run;

pub use args::{build_args, HwKind};
pub use outpath::{plan_output, temp_path, Overwrite, PlanError};
pub use preset::{builtin_presets, AudioCodec, AudioSpec, Preset, PresetError, RateControl, VideoCodec, VideoSpec};
pub use run::{detect_hw, ffmpeg_version, preview_frame, run_encode, Progress, RunError};
