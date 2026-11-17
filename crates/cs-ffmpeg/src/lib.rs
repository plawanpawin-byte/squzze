//! Presets, FFmpeg argument building, process running and hardware detection.

pub mod args;
pub mod audio;
pub mod encoders;
pub mod outpath;
pub mod preset;
pub mod run;
pub mod verify;

pub use args::{build_args, build_args_full, build_args_pass, build_args_trim, strip_metadata, HwKind, Trim};
pub use audio::{AudioChoice, AudioMode};
pub use outpath::{plan_output, temp_path, Overwrite, PlanError};
pub use preset::{builtin_presets, Speed, PRESET_SCHEMA, AudioCodec, AudioSpec, Preset, PresetError, RateControl, VideoCodec, VideoSpec};
pub use run::{detect_hw, ffmpeg_version, preview_frame, run_encode, run_encode_logged, Progress, RunError};
pub use verify::{sha256_file, verify_output, verify_output_with, Verify};
pub use encoders::{list_encoders, parse_encoders, unavailable_reason};
