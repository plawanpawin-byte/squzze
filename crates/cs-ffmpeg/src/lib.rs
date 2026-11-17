//! Presets, FFmpeg argument building, process running and hardware detection.

pub mod args;
pub mod encoders;
pub mod outpath;
pub mod preset;
pub mod run;
pub mod verify;

pub use args::{build_args, build_args_trim, HwKind, Trim};
pub use outpath::{plan_output, temp_path, Overwrite, PlanError};
pub use preset::{builtin_presets, AudioCodec, AudioSpec, Preset, PresetError, RateControl, VideoCodec, VideoSpec};
pub use run::{detect_hw, ffmpeg_version, preview_frame, run_encode, run_encode_logged, Progress, RunError};
pub use verify::{verify_output, Verify};
pub use encoders::{list_encoders, parse_encoders, unavailable_reason};
