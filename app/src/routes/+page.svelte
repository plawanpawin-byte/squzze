<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { app, type Tab } from "$lib/app.svelte";
  import type { Preset } from "$lib/types";
  import PresetTree from "../components/PresetTree.svelte";
  import Preview from "../components/Preview.svelte";
  import BatchView from "../components/BatchView.svelte";
  import QueueView from "../components/QueueView.svelte";
  import Inspector from "../components/Inspector.svelte";
  import PresetDialog from "../components/PresetDialog.svelte";

  let dialog = $state<{ initial: Preset | null } | null>(null);
  const tabs: { id: Tab; label: string }[] = [
    { id: "current", label: "Current" },
    { id: "active", label: "Active" },
    { id: "completed", label: "Completed" },
  ];
  const count = $derived({ current: app.sources.length, active: app.activeJobs.length, completed: app.finishedJobs.length });

  onMount(() => {
    app.init().catch((e) => app.say(String(e), "error"));
  });

  function key(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (e.ctrlKey && e.key.toLowerCase() === "o") { e.preventDefault(); app.pickAndAdd(); }
    else if (e.ctrlKey && e.key === "Enter") { e.preventDefault(); app.startBatch(); }
    else if (e.key === "Delete" && app.tab === "current" && app.selected && tag !== "INPUT" && tag !== "SELECT") app.removeSource(app.selected);
  }
</script>

<svelte:window onkeydown={key} />

<div class="app">
  <header class="top">
    <div class="brand">Squeeze</div>
    <div class="tabs" role="tablist" aria-label="Queue views">
      {#each tabs as t}
        <button role="tab" aria-selected={app.tab === t.id} class:on={app.tab === t.id} onclick={() => (app.tab = t.id)}>
          {t.label}{count[t.id] ? ` (${count[t.id]})` : ""}
        </button>
      {/each}
    </div>
    <button class="ghost" onclick={() => app.pickAndAdd()}>Add files…</button>
  </header>

  <PresetTree onnew={() => (dialog = { initial: null })} onedit={(p) => (dialog = { initial: p })} />

  <main class="center">
    {#if app.ffmpegMissing}
      <div class="warn" role="alert">
        FFmpeg was not found, so nothing can be converted yet. Install it with <code>winget install Gyan.FFmpeg</code>, or put
        <code>ffmpeg.exe</code> and <code>ffprobe.exe</code> in a folder named <code>ffmpeg</code> next to Squeeze, then restart.
      </div>
    {/if}
    {#if app.tab === "current"}
      <Preview />
      <BatchView />
    {:else if app.tab === "active"}
      <QueueView jobs={app.activeJobs} kind="active" />
    {:else}
      <QueueView jobs={app.finishedJobs} kind="completed" />
    {/if}
  </main>

  <Inspector />

  <footer class="bottom">
    {#if app.settings}
      <label>Run
        <select value={app.settings.max_parallel} onchange={(e) => app.setSetting({ max_parallel: Number(e.currentTarget.value) })} aria-label="Jobs at the same time">
          {#each [1, 2, 3, 4] as n}<option value={n}>{n} at a time</option>{/each}
        </select>
      </label>
      <label class="hwl" title={app.info?.hw.length ? app.info.hw.join(", ") : "No working hardware encoder found"}>
        <input type="checkbox" checked={app.settings.use_hw} onchange={(e) => app.setSetting({ use_hw: e.currentTarget.checked })} />
        Use GPU {app.info?.hw_ready ? (app.info.hw.length ? `(${app.info.hw.join(", ")})` : "(none found)") : "(checking…)"}
      </label>
    {/if}
    <span class="msg" class:error={app.messageKind === "error"} role="status" aria-live="polite">{app.message}</span>
    <span class="spacer"></span>
    {#if app.info}<span class="ver">v{app.info.version}</span>{/if}
    <button class="primary" disabled={!app.canStart || app.ffmpegMissing} onclick={() => app.startBatch()}>Start batch</button>
  </footer>
</div>

{#if dialog}
  <PresetDialog initial={dialog.initial} onclose={() => (dialog = null)} />
{/if}

<style>
  .app {
    display: grid;
    height: 100vh;
    grid-template-columns: 250px 1fr 270px;
    grid-template-rows: 44px 1fr 48px;
    grid-template-areas: "top top top" "side center insp" "bottom bottom bottom";
  }
  .top { grid-area: top; display: flex; align-items: center; gap: 16px; padding: 0 14px; background: var(--panel); border-bottom: 1px solid var(--line); }
  .brand { font-weight: 700; color: var(--accent); }
  .tabs { margin: 0 auto; display: flex; background: var(--panel-2); border-radius: 8px; padding: 2px; }
  .tabs button { background: none; border: 0; padding: 5px 16px; border-radius: 6px; color: var(--muted); }
  .tabs button.on { background: var(--bg); color: var(--text); }
  .ghost { background: none; border: 1px solid var(--line); border-radius: 6px; padding: 5px 12px; }
  .center { grid-area: center; display: grid; grid-template-rows: auto 1fr; min-width: 0; min-height: 0; overflow: hidden; }
  .warn { grid-column: 1; background: #5a2a1f; color: #ffd9cf; padding: 8px 14px; }
  .bottom { grid-area: bottom; display: flex; align-items: center; gap: 14px; padding: 0 14px; background: var(--panel); border-top: 1px solid var(--line); }
  .spacer { flex: 1; }
  .msg { color: var(--muted); }
  .msg.error { color: var(--danger); }
  .ver { color: var(--muted); }
  .hwl { display: flex; align-items: center; gap: 6px; }
  select { background: var(--panel-2); color: var(--text); border: 1px solid var(--line); border-radius: 6px; padding: 4px 8px; margin-left: 6px; }
  .primary { background: var(--accent); color: var(--accent-ink); border: 0; border-radius: 6px; padding: 7px 18px; font-weight: 600; }
  .primary:disabled { opacity: 0.4; cursor: default; }
  code { background: #0004; padding: 0 4px; border-radius: 3px; }
</style>
