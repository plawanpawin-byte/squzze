export type RateControl = { mode: "crf" | "bitrate"; value: number };
export type VideoSpec = { codec: string; rate: RateControl; max_height: number | null; fps: number | null };
export type AudioSpec = { codec: string; bitrate_kbps: number | null; channels: number | null };
export type Preset = { id: string; name: string; group: string; container: string; video: VideoSpec | null; audio: AudioSpec | null };
export type VideoInfo = { codec: string; width: number; height: number; fps: number | null; pix_fmt: string | null; field_order: string | null };
export type AudioInfo = { codec: string; channels: number; sample_rate: number | null };
export type MediaInfo = {
  path: string; file_name: string; format_name: string; duration_secs: number | null;
  size_bytes: number | null; bit_rate: number | null; video: VideoInfo | null; audio: AudioInfo | null;
};
export type JobState = "queued" | "running" | "done" | "failed" | "cancelled";
export type Job = {
  id: number; source: string; file_name: string; duration_secs: number | null; preset: Preset;
  dest_dir: string | null; state: JobState; progress: number; fps: number | null; speed: number | null;
  eta_secs: number | null; error: string | null; output: string | null; note: string | null; attempt: number;
};
export type Settings = { max_parallel: number; use_hw: boolean; default_dest: string | null; template: string; overwrite: "rename" | "replace" | "skip" };
export type AppInfo = { version: string; ffmpeg_path: string | null; ffmpeg_version: string | null; hw: string[]; hw_ready: boolean };
export type ProbeResult = { path: string; info: MediaInfo | null; error: string | null };
export type QueueEvent = { kind: "updated"; job: Job } | { kind: "removed"; id: number } | { kind: "idle" };
