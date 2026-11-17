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
  const count = $derived({ current: 0, active: app.activeJobs.length, completed: 0 });
  const cols = $derived(`${app.showLeft ? "240px" : "0px"} minmax(0, 1fr) ${app.showRight ? "300px" : "0px"}`);

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

<div class="app" style:grid-template-columns={cols}>
  <header class="top">
    <div class="left">
      <button class="icon-btn" class:on={app.showLeft} aria-pressed={app.showLeft} aria-label="Show or hide the preset sidebar" title="Sidebar" onclick={() => (app.showLeft = !app.showLeft)}>
        <svg width="18" height="14" viewBox="0 0 18 14" aria-hidden="true"><rect x="0.5" y="0.5" width="17" height="13" rx="2.5" fill="none" stroke="currentColor"/><rect x="2" y="2" width="5" height="10" rx="1" fill="currentColor"/></svg>
      </button>
      <span class="brand">Squeeze</span>
    </div>
    <div class="seg tabs" role="tablist" aria-label="Queue views">
      {#each tabs as t}
        <button role="tab" aria-selected={app.tab === t.id} onclick={() => (app.tab = t.id)}>
          {t.label}{count[t.id] ? ` · ${count[t.id]}` : ""}
        </button>
      {/each}
    </div>
    <div class="right">
      <button class="btn" onclick={() => app.pickAndAdd()} title="Add files (Ctrl+O)">Add files…</button>
      <button class="icon-btn" class:on={app.showRight} aria-pressed={app.showRight} aria-label="Show or hide the inspector" title="Inspector" onclick={() => (app.showRight = !app.showRight)}>
        <svg width="18" height="14" viewBox="0 0 18 14" aria-hidden="true"><rect x="0.5" y="0.5" width="17" height="13" rx="2.5" fill="none" stroke="currentColor"/><rect x="11" y="2" width="5" height="10" rx="1" fill="currentColor"/></svg>
      </button>
    </div>
  </header>

  <div class="pane-left" class:hidden={!app.showLeft}>
    <PresetTree onnew={() => (dialog = { initial: null })} onedit={(p) => (dialog = { initial: p })} />
  </div>

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

  <div class="pane-right" class:hidden={!app.showRight}>
    <Inspector />
  </div>

  <footer class="bottom">
    <div class="b-left">
      <button class="icon-btn plus" aria-label="Add files" title="Add files (Ctrl+O)" onclick={() => app.pickAndAdd()}>＋</button>
      <span class="msg" class:error={app.messageKind === "error"} role="status" aria-live="polite">{app.message}</span>
    </div>
    <div class="b-right">
      {#if app.settings}
        <label class="hwl" title={app.info?.hw.length ? app.info.hw.join(", ") : "No working hardware encoder found"}>
          <input type="checkbox" checked={app.settings.use_hw} onchange={(e) => app.setSetting({ use_hw: e.currentTarget.checked })} />
          GPU {app.info?.hw_ready ? (app.info.hw.length ? app.info.hw.join(" · ") : "none") : "checking…"}
        </label>
        <label>Jobs
          <select class="field" value={app.settings.max_parallel} onchange={(e) => app.setSetting({ max_parallel: Number(e.currentTarget.value) })} aria-label="Jobs at the same time">
            {#each [1, 2, 3, 4] as n}<option value={n}>{n} at a time</option>{/each}
          </select>
        </label>
      {/if}
      <label>Process on
        <select class="field" aria-label="Process on"><option>This Computer</option></select>
      </label>
      <button class="btn primary" disabled={!app.canStart || app.ffmpegMissing} onclick={() => app.startBatch()}>Start batch</button>
    </div>
  </footer>
</div>

{#if dialog}
  <PresetDialog initial={dialog.initial} onclose={() => (dialog = null)} />
{/if}

<style>
  .app {
    display: grid;
    height: 100vh;
    grid-template-rows: 40px minmax(0, 1fr) 36px;
    grid-template-areas: "top top top" "side center insp" "bottom bottom bottom";
    transition: grid-template-columns 0.15s ease;
  }
  .top {
    grid-area: top; display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; padding: 0 10px;
    background: var(--panel); border-bottom: 1px solid #000a; box-shadow: 0 1px 0 var(--line-soft);
  }
  .left, .right { display: flex; align-items: center; gap: 10px; }
  .right { justify-content: flex-end; }
  .brand { font-weight: 600; color: var(--muted); letter-spacing: 0.02em; }
  .icon-btn.on { color: var(--accent); }
  .tabs button { min-width: 92px; }
  .pane-left { grid-area: side; min-width: 0; overflow: hidden; border-right: 1px solid #000a; }
  .pane-right { grid-area: insp; min-width: 0; overflow: hidden; border-left: 1px solid #000a; }
  .hidden { visibility: hidden; border: 0; }
  .center { grid-area: center; display: flex; flex-direction: column; min-width: 0; min-height: 0; overflow: hidden; background: var(--bg); }
  .warn { background: #4a2a12; color: #ffd8b0; padding: 6px 12px; border-bottom: 1px solid #000a; }
  .bottom {
    grid-area: bottom; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 0 10px;
    background: var(--panel); border-top: 1px solid #000a;
  }
  .b-left, .b-right { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .b-right label { display: flex; align-items: center; gap: 6px; color: var(--muted); }
  .plus { font-size: 15px; }
  .msg { color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .msg.error { color: var(--danger); }
  code { background: #0004; padding: 0 4px; border-radius: 3px; }
</style>
