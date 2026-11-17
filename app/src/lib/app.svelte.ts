import { api, inTauri, onFileDrop, onHwReady, onQueue } from "./api";
import { baseName } from "./format";
import type { AppInfo, Job, MediaInfo, Preset, Settings } from "./types";

export type Tab = "current" | "active" | "completed";
export type PendingOutput = { uid: number; presetId: string };
export type Source = { path: string; name: string; info: MediaInfo | null; error: string | null; outputs: PendingOutput[] };

let uid = 1;

class App {
  info = $state<AppInfo | null>(null);
  presets = $state<Preset[]>([]);
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
  private timer: ReturnType<typeof setTimeout> | undefined;

  current = $derived(this.sources.find((s) => s.path === this.selected) ?? null);
  activeJobs = $derived(this.jobs.filter((j) => j.state === "running" || j.state === "queued"));
  finishedJobs = $derived(this.jobs.filter((j) => j.state === "done" || j.state === "failed" || j.state === "cancelled"));
  job = $derived(this.jobs.find((j) => j.id === this.selectedJob) ?? null);
  canStart = $derived(!this.busy && this.sources.some((s) => s.info && s.outputs.length > 0));
  ffmpegMissing = $derived(this.info !== null && this.info.ffmpeg_path === null);

  say(text: string, kind: "info" | "error" = "info") {
    this.message = text;
    this.messageKind = kind;
    clearTimeout(this.timer);
    if (text) this.timer = setTimeout(() => (this.message = ""), kind === "error" ? 9000 : 4000);
  }

  preset(id: string): Preset | undefined {
    return this.presets.find((p) => p.id === id);
  }

  async init() {
    if (!inTauri) return;
    const [info, presets, settings, jobs] = await Promise.all([api.appInfo(), api.listPresets(), api.getSettings(), api.queueList()]);
    this.info = info;
    this.presets = presets;
    this.settings = settings;
    this.jobs = jobs;
    await onQueue((e) => {
      if (e.kind === "updated") {
        const i = this.jobs.findIndex((j) => j.id === e.job.id);
        if (i >= 0) this.jobs[i] = e.job;
        else this.jobs.push(e.job);
      } else if (e.kind === "removed") {
        this.jobs = this.jobs.filter((j) => j.id !== e.id);
        if (this.selectedJob === e.id) this.selectedJob = null;
      } else if (e.kind === "idle") {
        this.say("Queue finished");
      }
    });
    await onHwReady(async () => (this.info = await api.appInfo()));
    await onFileDrop((paths) => this.addPaths(paths));
  }

  defaultPresetFor(info: MediaInfo | null): string {
    const wanted = info && !info.video ? "aac-256" : "h264-1080";
    return this.presets.some((p) => p.id === wanted) ? wanted : (this.presets[0]?.id ?? "");
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
    try {
      const results = await api.probeFiles(fresh.slice(0, 300));
      let failed = 0;
      for (const r of results) {
        const src: Source = { path: r.path, name: baseName(r.path), info: r.info, error: r.error, outputs: [] };
        if (r.info) src.outputs.push({ uid: uid++, presetId: this.defaultPresetFor(r.info) });
        else failed++;
        this.sources.push(src);
      }
      if (!this.selected) this.selected = this.sources[0]?.path ?? null;
      if (failed) this.say(`${failed} file(s) could not be read — see the list`, "error");
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
    for (const s of this.sources) {
      if (!s.info) { remaining.push(s); continue; }
      for (const o of s.outputs) {
        try {
          await api.addJob(s.path, o.presetId, this.dest);
          ok++;
        } catch (e) {
          errors.push(`${s.name}: ${e}`);
        }
      }
    }
    this.sources = remaining.filter((s) => s.error);
    this.selected = this.sources[0]?.path ?? null;
    if (ok) {
      await api.queueStart();
      this.tab = "active";
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

  async setSetting(patch: Partial<Settings>) {
    if (!this.settings) return;
    const next = await this.run(() => api.setSettings({ ...this.settings!, ...patch }));
    if (next) this.settings = next;
  }
}

export const app = new App();
