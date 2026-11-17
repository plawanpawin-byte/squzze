import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { AppInfo, Job, Preset, ProbeResult, QueueEvent, Settings } from "./types";

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  listPresets: () => invoke<Preset[]>("list_presets"),
  saveCustomPreset: (preset: Preset) => invoke<Preset[]>("save_custom_preset", { preset }),
  deleteCustomPreset: (id: string) => invoke<Preset[]>("delete_custom_preset", { id }),
  exportPreset: (id: string) => invoke<boolean>("export_preset", { id }),
  importPreset: () => invoke<Preset[]>("import_preset"),
  pickFiles: () => invoke<string[]>("pick_files"),
  pickFolder: () => invoke<string | null>("pick_folder"),
  probeFiles: (paths: string[]) => invoke<ProbeResult[]>("probe_files", { paths }),
  preview: (path: string, secs: number) => invoke<string>("preview", { path, secs }),
  addJob: (source: string, presetId: string, destDir: string | null) =>
    invoke<number>("add_job", { source, presetId, destDir }),
  queueList: () => invoke<Job[]>("queue_list"),
  queueStart: () => invoke<void>("queue_start"),
  queuePause: () => invoke<void>("queue_pause"),
  jobCancel: (id: number) => invoke<void>("job_cancel", { id }),
  queueCancelAll: () => invoke<void>("queue_cancel_all"),
  jobRetry: (id: number) => invoke<void>("job_retry", { id }),
  jobRemove: (id: number) => invoke<boolean>("job_remove", { id }),
  queueClearFinished: () => invoke<void>("queue_clear_finished"),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<Settings>("set_settings", { settings }),
  revealOutput: (id: number) => invoke<void>("reveal_output", { id }),
};

export function onQueue(cb: (e: QueueEvent) => void): Promise<UnlistenFn> {
  return listen<QueueEvent>("queue", (e) => cb(e.payload));
}
export function onHwReady(cb: () => void): Promise<UnlistenFn> {
  return listen("hw-ready", () => cb());
}
export function onFileDrop(cb: (paths: string[]) => void): Promise<UnlistenFn> {
  return getCurrentWebview().onDragDropEvent((e) => {
    if (e.payload.type === "drop") cb(e.payload.paths);
  });
}
