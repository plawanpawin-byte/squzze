export type RateControl = { mode: "crf" | "bitrate"; value: number };
export type VideoSpec = {
  codec: string; rate: RateControl; max_height: number | null; fps: number | null;
  bit_depth?: number | null; keyint_secs?: number | null; speed?: "fast" | "medium" | "slow" | null;
  two_pass?: boolean; profile?: string | null; level?: string | null;
};
export type AudioSpec = { codec: string; bitrate_kbps: number | null; channels: number | null };
export type Preset = { schema?: number; description?: string | null; id: string; name: string; group: string; container: string; video: VideoSpec | null; audio: AudioSpec | null };
export type VideoInfo = {
  codec: string; width: number; height: number; fps: number | null; pix_fmt: string | null; field_order: string | null;
  profile: string | null; level: string | null; bit_depth: number | null; color_primaries: string | null;
  color_transfer: string | null; color_space: string | null; color_range: string | null;
  vfr: boolean; hdr: boolean; timecode: string | null;
};
export type AudioInfo = {
  codec: string; channels: number; sample_rate: number | null; index: number;
  channel_layout: string | null; bit_rate: number | null; language: string | null; title: string | null;
};
export type SubtitleInfo = { codec: string; language: string | null; title: string | null };
export type AudioMode = "encode" | "copy" | "none";
export type AudioChoice = { stream: number | null; mode: AudioMode; channels: number | null };
export type MediaInfo = {
  path: string; file_name: string; format_name: string; duration_secs: number | null;
  size_bytes: number | null; bit_rate: number | null; video: VideoInfo | null; audio: AudioInfo | null;
  audio_streams: AudioInfo[]; subtitles: SubtitleInfo[];
};
export type JobState = "queued" | "running" | "done" | "failed" | "cancelled";
export type Job = {
  id: number; source: string; file_name: string; duration_secs: number | null; preset: Preset;
  dest_dir: string | null; state: JobState; progress: number; fps: number | null; speed: number | null;
  eta_secs: number | null; error: string | null; output: string | null; note: string | null; attempt: number;
  start_secs: number | null; end_secs: number | null; source_size: number | null;
  started_ms: number | null; finished_ms: number | null; cmdline: string | null; log: string[]; verify: Verify | null;
  audio: AudioChoice; strip_metadata?: boolean;
};
export type Verify = {
  ok: boolean; notes: string[]; size_bytes: number | null; duration_secs: number | null;
  video_codec: string | null; audio_codec: string | null; width: number | null; height: number | null;
  sha256?: string | null;
};
export type Settings = { max_parallel: number; use_hw: boolean; default_dest: string | null; template: string; overwrite: "rename" | "replace" | "skip"; favorites: string[]; strip_metadata: boolean };
export type AppInfo = { version: string; ffmpeg_path: string | null; ffmpeg_version: string | null; hw: string[]; hw_ready: boolean };
export type ProbeResult = { path: string; info: MediaInfo | null; error: string | null };
export type QueueEvent = { kind: "updated"; job: Job } | { kind: "removed"; id: number } | { kind: "idle" };
