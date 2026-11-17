import { api, inTauri, onFileDrop, onHwReady, onQueue } from "./api";
import { baseName, fmtBytes, fmtDuration } from "./format";
import type { Preset as PresetT } from "./types";
import type { AppInfo, AudioChoice, Job, MediaInfo, Preset, Settings } from "./types";

export type Tab = "current" | "active" | "completed";
export type PendingOutput = { uid: number; presetId: string };
export type Source = {
  path: string;
  name: string;
  info: MediaInfo | null;
  error: string | null;
  outputs: PendingOutput[];
  /** In / out points in seconds (null = start / end of file). */
  inPt: number | null;
  outPt: number | null;
  /** Which audio stream, copy/convert/remove, downmix — applies to every output of this file. */
  audio: AudioChoice;
};

export const defaultAudio = (): AudioChoice => ({ stream: null, mode: "encode", channels: null });

/** Mirrors cs_ffmpeg::audio::copy_ok so problems show before a job is queued. */
export function copyOk(container: string, codec: string): boolean {
  const pcm = codec.startsWith("pcm_");
  switch (container) {
    case "mkv": return true;
    case "mp4": return ["aac", "mp3", "opus", "ac3", "eac3", "flac"].includes(codec);
    case "mov": return ["aac", "mp3", "ac3"].includes(codec) || pcm;
    case "webm": return ["opus", "vorbis"].includes(codec);
    case "m4a": return codec === "aac";
    case "opus": return codec === "opus";
    case "flac": return codec === "flac";
    case "wav": return pcm;
    case "mp3": return codec === "mp3";
    default: return false;
  }
}

let uid = 1;

const LAYOUT_KEY = "squeeze.layout.v1";
export const defaultPreviewH = () => Math.round(Math.min(440, Math.max(170, (typeof window === "undefined" ? 800 : window.innerHeight) * 0.36)));
export const LIMITS = { left: [170, 440], right: [230, 560], previewMin: 110 } as const;

type Layout = { leftW: number; rightW: number; previewH: number | null; showLeft: boolean; showRight: boolean };

function loadLayout(): Layout {
  const d: Layout = { leftW: 250, rightW: 310, previewH: null, showLeft: true, showRight: true };
  try {
    const raw = localStorage.getItem(LAYOUT_KEY);
    if (!raw) return d;
    const v = JSON.parse(raw) as Partial<Layout>;
    const n = (x: unknown, lo: number, hi: number, f: number) => (typeof x === "number" && x >= lo && x <= hi ? x : f);
    return {
      leftW: n(v.leftW, LIMITS.left[0], LIMITS.left[1], d.leftW),
      rightW: n(v.rightW, LIMITS.right[0], LIMITS.right[1], d.rightW),
      previewH: v.previewH == null ? null : n(v.previewH, LIMITS.previewMin, 2000, 300),
      showLeft: v.showLeft !== false,
      showRight: v.showRight !== false,
    };
  } catch {
    return d; // storage can be unavailable; the layout just will not persist
  }
}

class App {
  info = $state<AppInfo | null>(null);
  presets = $state<Preset[]>([]);
  /** preset id -> why the installed FFmpeg cannot run it */
  unavailable = $state<Record<string, string>>({});
  settings = $state<Settings | null>(null);
  jobs = $state<Job[]>([]);
  sources = $state<Source[]>([]);
  selected = $state<string | null>(null);
  selectedJob = $state<number | null>(null);
  tab = $state<Tab>("current");
  dest = $state<string | null>(null);
  applyAll = $state(false);
  busy = $state(false);
  message = $state("");
  messageKind = $state<"info" | "error">("info");
  presetFilter = $state("");
  selectedOutput = $state<number | null>(null);
  thumbs = $state<Record<string, string>>({});
  dragging = $state(false);
  now = $state(Date.now());

  private L = loadLayout();
  leftW = $state(this.L.leftW);
  rightW = $state(this.L.rightW);
  previewH = $state<number | null>(this.L.previewH);
  showLeft = $state(this.L.showLeft);
  showRight = $state(this.L.showRight);

  /** Job ids started by the most recent "Start batch" (for the overall progress bar). */
  runIds = $state<number[]>([]);
  private timer: ReturnType<typeof setTimeout> | undefined;

  current = $derived(this.sources.find((s) => s.path === this.selected) ?? null);
  activeJobs = $derived(this.jobs.filter((j) => j.state === "running" || j.state === "queued"));
  finishedJobs = $derived(this.jobs.filter((j) => j.state === "done" || j.state === "failed" || j.state === "cancelled"));
  job = $derived(this.jobs.find((j) => j.id === this.selectedJob) ?? null);
  ffmpegMissing = $derived(this.info !== null && this.info.ffmpeg_path === null);
  outputCount = $derived(this.sources.reduce((n, s) => n + (s.info ? s.outputs.length : 0), 0));
  /** First audio setting that cannot work with the chosen outputs (same rules as the backend). */
  audioProblem = $derived.by(() => {
    for (const s of this.sources) {
      if (!s.info) continue;
      const a = s.audio, streams = s.info.audio_streams;
      for (const o of s.outputs) {
        const p = this.preset(o.presetId);
        if (!p) continue;
        if (a.mode === "none" && !p.video) return `${s.name}: "${p.name}" is audio-only, so the audio cannot be removed`;
        if (a.mode === "copy") {
          const st = streams[a.stream ?? 0];
          if (!st) return `${s.name}: there is no audio to copy`;
          if (!copyOk(p.container, st.codec)) return `${s.name}: the original ${st.codec} audio cannot be copied into .${p.container} — choose Convert`;
        }
      }
    }
    return "";
  });
  startHint = $derived.by(() => {
    if (this.ffmpegMissing) return "FFmpeg was not found";
    if (this.audioProblem) return this.audioProblem;
    if (this.busy) return "Adding jobs…";
    if (this.sources.length === 0) return "Add a file first";
    if (!this.sources.some((s) => s.info)) return "None of the files could be read";
    if (this.outputCount === 0) return "Pick at least one preset for a file";
    return "";
  });
  canStart = $derived(this.startHint === "");

  /** 0–100 over the jobs of the current run. */
  overall = $derived.by(() => {
    const js = this.jobs.filter((j) => this.runIds.includes(j.id));
    if (!js.length) return null;
    const w = (j: Job) => Math.max(j.duration_secs ?? 1, 1);
    const total = js.reduce((a, j) => a + w(j), 0);
    const done = js.reduce((a, j) => a + w(j) * ((j.state === "done" ? 100 : j.state === "running" ? Math.min(j.progress, 99) : 0) / 100), 0);
    const finished = js.filter((j) => j.state === "done" || j.state === "failed" || j.state === "cancelled").length;
    // never claim 100% until every job of the run has actually finished
    const pct = finished === js.length ? 100 : Math.min(99, Math.floor((done / total) * 100));
    return { percent: pct, finished, total: js.length };
  });

  say(text: string, kind: "info" | "error" = "info") {
    this.message = text;
    this.messageKind = kind;
    clearTimeout(this.timer);
    if (text) this.timer = setTimeout(() => (this.message = ""), kind === "error" ? 9000 : 4000);
  }

  preset(id: string): Preset | undefined {
    return this.presets.find((p) => p.id === id);
  }

  saveLayout() {
    try {
      localStorage.setItem(
        LAYOUT_KEY,
        JSON.stringify({ leftW: this.leftW, rightW: this.rightW, previewH: this.previewH, showLeft: this.showLeft, showRight: this.showRight }),
      );
    } catch {
      /* not persisted */
    }
  }
  resetLayout() {
    this.leftW = 250;
    this.rightW = 310;
    this.previewH = null;
    this.showLeft = this.showRight = true;
    this.saveLayout();
  }

  /** Small poster frame for the batch list, loaded once per source. */
  async loadThumb(path: string, secs: number) {
    if (path in this.thumbs) return;
    this.thumbs[path] = "";
    try {
      this.thumbs[path] = await api.preview(path, secs);
    } catch {
      /* no thumbnail; the block shows a placeholder */
    }
  }

  addDefaultOutput(path: string) {
    const s = this.sources.find((x) => x.path === path);
    if (s?.info) s.outputs.push({ uid: uid++, presetId: this.defaultPresetFor(s.info) });
  }

  async init() {
    if (!inTauri) return;
    const [info, presets, settings, jobs] = await Promise.all([api.appInfo(), api.listPresets(), api.getSettings(), api.queueList()]);
    this.info = info;
    this.presets = presets;
    this.settings = settings;
    this.jobs = jobs;
    this.unavailable = await api.presetAvailability();
    setInterval(() => (this.now = Date.now()), 1000);
    await onQueue((e) => {
      if (e.kind === "updated") {
        const i = this.jobs.findIndex((j) => j.id === e.job.id);
        if (i >= 0) this.jobs[i] = e.job;
        else this.jobs.push(e.job);
      } else if (e.kind === "removed") {
        this.jobs = this.jobs.filter((j) => j.id !== e.id);
        if (this.selectedJob === e.id) this.selectedJob = null;
      } else if (e.kind === "idle") {
        const o = this.overall;
        const failed = this.jobs.filter((j) => this.runIds.includes(j.id) && j.state === "failed").length;
        this.say(failed ? `Queue finished — ${failed} job(s) failed` : o ? `Queue finished — ${o.total} job(s) done` : "Queue finished", failed ? "error" : "info");
      }
    });
    await onHwReady(async () => {
      this.info = await api.appInfo();
      this.unavailable = await api.presetAvailability();
    });
    await onFileDrop((paths) => this.addPaths(paths), (over) => (this.dragging = over));
  }

  defaultPresetFor(info: MediaInfo | null): string {
    const wanted = info && !info.video ? "aac-256" : "h264-1080";
    const ok = (id: string) => this.presets.some((p) => p.id === id) && !this.unavailable[id];
    if (ok(wanted)) return wanted;
    return this.presets.find((p) => !this.unavailable[p.id] && (info?.video ? true : !p.video))?.id ?? (this.presets[0]?.id ?? "");
  }

  async pickAndAdd() {
    try {
      const paths = await api.pickFiles();
      if (paths.length) await this.addPaths(paths);
    } catch (e) {
      this.say(String(e), "error");
    }
  }

  async addPaths(paths: string[]) {
    const fresh = [...new Set(paths)].filter((p) => !this.sources.some((s) => s.path === p));
    if (!fresh.length) return;
    this.tab = "current";
    this.say(`Reading ${fresh.length} file(s)…`);
    try {
      const results = await api.probeFiles(fresh.slice(0, 300));
      let failed = 0;
      for (const r of results) {
        const src: Source = { path: r.path, name: baseName(r.path), info: r.info, error: r.error, outputs: [], inPt: null, outPt: null, audio: defaultAudio() };
        if (r.info) src.outputs.push({ uid: uid++, presetId: this.defaultPresetFor(r.info) });
        else failed++;
        this.sources.push(src);
      }
      if (!this.selected) this.selected = this.sources[0]?.path ?? null;
      const ok = results.length - failed;
      if (failed) this.say(`${ok} added, ${failed} could not be read — see the list`, "error");
      else this.say(`${ok} file(s) added`);
    } catch (e) {
      this.say(String(e), "error");
    }
  }

  removeSource(path: string) {
    this.sources = this.sources.filter((s) => s.path !== path);
    if (this.selected === path) this.selected = this.sources[0]?.path ?? null;
  }

  addOutput(presetId: string) {
    const targets = this.applyAll ? this.sources.filter((s) => s.info) : this.current?.info ? [this.current] : [];
    if (!targets.length) return this.say("Add a file first, then pick a preset", "error");
    if (this.unavailable[presetId]) return this.say(this.unavailable[presetId], "error");
    for (const s of targets) {
      const p = this.preset(presetId);
      if (p?.video && s.info && !s.info.video) continue; // video preset on an audio-only source
      s.outputs.push({ uid: uid++, presetId });
    }
  }

  removeOutput(path: string, id: number) {
    const s = this.sources.find((x) => x.path === path);
    if (s) s.outputs = s.outputs.filter((o) => o.uid !== id);
  }

  setIn(secs: number) {
    const s = this.current;
    if (!s?.info) return;
    const d = s.info.duration_secs ?? Infinity;
    s.inPt = Math.min(Math.max(0, secs), d);
    if (s.outPt != null && s.outPt <= s.inPt) s.outPt = null;
  }
  setOut(secs: number) {
    const s = this.current;
    if (!s?.info) return;
    const d = s.info.duration_secs ?? Infinity;
    const v = Math.min(Math.max(0, secs), d);
    if (v <= (s.inPt ?? 0)) return this.say("The out point must be after the in point", "error");
    s.outPt = v;
  }
  clearTrim() {
    const s = this.current;
    if (s) {
      s.inPt = null;
      s.outPt = null;
    }
  }

  async chooseDest() {
    try {
      const d = await api.pickFolder();
      if (d) this.dest = d;
    } catch (e) {
      this.say(String(e), "error");
    }
  }

  async startBatch() {
    if (!this.canStart) return;
    this.busy = true;
    let ok = 0;
    const errors: string[] = [];
    const remaining: Source[] = [];
    const ids: number[] = [];
    for (const s of this.sources) {
      if (!s.info) { remaining.push(s); continue; }
      for (const o of s.outputs) {
        try {
          ids.push(await api.addJob(s.path, o.presetId, this.dest, s.inPt, s.outPt, s.audio));
          ok++;
        } catch (e) {
          errors.push(`${s.name}: ${e}`);
        }
      }
    }
    this.sources = remaining.filter((s) => s.error);
    this.selected = this.sources[0]?.path ?? null;
    if (ok) {
      this.runIds = ids;
      await api.queueStart();
      this.tab = "active";
      this.say(`${ok} job(s) started`);
    }
    if (errors.length) this.say(errors.slice(0, 2).join(" · "), "error");
    this.busy = false;
  }

  async run<T>(f: () => Promise<T>): Promise<T | undefined> {
    try {
      return await f();
    } catch (e) {
      this.say(String(e), "error");
    }
  }

  private settingsChain: Promise<unknown> = Promise.resolve();

  /** Changes show immediately; saves run one after another so quick successive edits never overwrite each other. */
  async setSetting(patch: Partial<Settings>) {
    if (!this.settings) return;
    this.settings = { ...this.settings, ...patch };
    const snapshot: Settings = { ...this.settings };
    const keys = Object.keys(patch) as (keyof Settings)[];
    this.settingsChain = this.settingsChain.then(async () => {
      try {
        const saved = await api.setSettings(snapshot);
        // adopt the server's cleaned-up value only for the fields this edit touched
        for (const k of keys) (this.settings as Settings)[k] = saved[k] as never;
      } catch (e) {
        this.say(String(e), "error");
        try {
          this.settings = await api.getSettings();
        } catch {
          /* keep what is shown */
        }
      }
    });
    await this.settingsChain;
  }

  /** Output size estimate in bytes, or null when it cannot be predicted (quality-based video). */
  estimate(preset: PresetT, info: MediaInfo, inPt: number | null, outPt: number | null, audio: AudioChoice): number | null {
    const dur = (outPt ?? info.duration_secs ?? 0) - (inPt ?? 0);
    if (!(dur > 0)) return null;
    let bits = 0;
    if (preset.video) {
      if (preset.video.rate.mode !== "bitrate") return null;
      bits += preset.video.rate.value * 1000 * dur;
    }
    const st = info.audio_streams[audio.stream ?? 0];
    if (audio.mode !== "none" && st) {
      if (audio.mode === "copy") {
        if (!st.bit_rate) return null;
        bits += st.bit_rate * dur;
      } else if (preset.audio) {
        if (preset.audio.codec === "pcm24") bits += 48000 * 24 * (audio.channels ?? preset.audio.channels ?? st.channels) * dur;
        else if (preset.audio.bitrate_kbps) bits += preset.audio.bitrate_kbps * 1000 * dur;
        else return null; // lossless FLAC: depends on the content
      }
    }
    return bits > 0 ? bits / 8 : null;
  }

  elapsed(j: Job): number | null {
    if (!j.started_ms) return null;
    const end = j.finished_ms ?? this.now;
    return Math.max(0, (end - j.started_ms) / 1000);
  }

  /** Plain-text report of a job, for support or records. */
  report(j: Job): string {
    const v = j.verify;
    const lines = [
      `Squeeze job report`,
      `File:        ${j.source}`,
      `Preset:      ${j.preset.name} (.${j.preset.container})`,
      `State:       ${j.state}${j.error ? ` — ${j.error}` : ""}`,
      j.start_secs != null || j.end_secs != null ? `Range:       ${fmtDuration(j.start_secs ?? 0)} – ${j.end_secs != null ? fmtDuration(j.end_secs) : "end"}` : "",
      j.output ? `Output:      ${j.output}` : "",
      j.source_size != null ? `Source size: ${fmtBytes(j.source_size)}` : "",
      v ? `Output size: ${fmtBytes(v.size_bytes)}  duration ${fmtDuration(v.duration_secs)}  ${v.video_codec ?? "no video"} ${v.width ? `${v.width}x${v.height}` : ""}  ${v.audio_codec ?? "no audio"}` : "",
      v ? `Verified:    ${v.ok ? "yes" : "NO — " + v.notes.join("; ")}` : "",
      v?.sha256 ? `SHA-256:     ${v.sha256}` : "",
      j.note ? `Note:        ${j.note}` : "",
      this.elapsed(j) != null ? `Elapsed:     ${fmtDuration(this.elapsed(j))}` : "",
      j.cmdline ? `\nCommand:\n${j.cmdline}` : "",
      j.log.length ? `\nFFmpeg output:\n${j.log.join("\n")}` : "",
    ];
    return lines.filter(Boolean).join("\n");
  }
}

export const app = new App();
