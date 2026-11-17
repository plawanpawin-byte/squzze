// Mock data for the UI prototype. Replaced by real data from the Rust core later.
export type PresetGroup = { name: string; items: string[] };
export type Output = { preset: string; location: string; filename: string };
export type Source = {
  name: string;
  info: string;
  duration: string;
  outputs: Output[];
};

export const presetGroups: PresetGroup[] = [
  { name: "Web", items: ["H.264 MP4 — 1080p", "H.264 MP4 — 720p", "HEVC MP4 — 1080p", "AV1 WebM — 1080p", "VP9 WebM — 1080p"] },
  { name: "Social", items: ["Vertical 1080×1920", "Square 1080×1080", "Short clip 30 s"] },
  { name: "Archive", items: ["ProRes 422 MOV", "ProRes 4444 MOV", "FFV1 MKV (lossless)"] },
  { name: "Audio", items: ["AAC 256 kbps", "Opus 128 kbps", "FLAC", "WAV 24-bit"] },
  { name: "Custom", items: [] },
];

export const sources: Source[] = [
  {
    name: "sample_a.mov",
    info: "ProRes 422 · 1920×1080 · 29.97 fps · 48 kHz stereo",
    duration: "00:00:30:00",
    outputs: [
      { preset: "H.264 MP4 — 1080p", location: "Same as source", filename: "sample_a-h264-1080p.mp4" },
      { preset: "AV1 WebM — 1080p", location: "Same as source", filename: "sample_a-av1-1080p.webm" },
    ],
  },
  {
    name: "sample_b.mp4",
    info: "H.264 · 3840×2160 · 30 fps · 48 kHz stereo",
    duration: "00:01:12:00",
    outputs: [
      { preset: "HEVC MP4 — 1080p", location: "D:\\Exports", filename: "sample_b-hevc-1080p.mp4" },
    ],
  },
];

export const inspectorFields: Record<string, [string, string][]> = {
  File: [
    ["Name", "sample_a.mov"],
    ["Container", "QuickTime / MOV"],
    ["Size", "412 MB"],
    ["Duration", "00:00:30:00"],
  ],
  Video: [
    ["Codec", "ProRes 422"],
    ["Resolution", "1920×1080"],
    ["Frame rate", "29.97 fps"],
    ["Field order", "Progressive"],
  ],
  Audio: [
    ["Codec", "PCM 24-bit"],
    ["Channels", "Stereo"],
    ["Sample rate", "48 kHz"],
  ],
};
