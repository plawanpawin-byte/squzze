<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { app, defaultPreviewH, LIMITS, type Tab } from "$lib/app.svelte";
  import type { Preset } from "$lib/types";
  import PresetTree from "../components/PresetTree.svelte";
  import Preview from "../components/Preview.svelte";
  import BatchView from "../components/BatchView.svelte";
  import QueueView from "../components/QueueView.svelte";
  import Inspector from "../components/Inspector.svelte";
  import PresetDialog from "../components/PresetDialog.svelte";
  import Splitter from "../components/Splitter.svelte";

  let dialog = $state<{ initial: Preset | null } | null>(null);
  let vh = $state(800);
  let vw = $state(1280);
  const tabs: { id: Tab; label: string }[] = [
    { id: "current", label: "Current" },
    { id: "active", label: "Active" },
    { id: "completed", label: "Completed" },
  ];
  const badge = $derived({ current: app.sources.length, active: app.activeJobs.length, completed: app.finishedJobs.length });
  const prevMax = $derived(Math.max(LIMITS.previewMin + 40, vh - 40 - 36 - 300));
  const frameH = $derived(app.previewH ?? defaultPreviewH());
  // Keep the centre usable on narrow windows even when the stored panel widths are large.
  const CENTER_MIN = 360;
  const leftEff = $derived(Math.max(LIMITS.left[0], Math.min(app.leftW, vw - (app.showRight ? LIMITS.right[0] : 0) - CENTER_MIN - 20)));
  const rightEff = $derived(Math.max(LIMITS.right[0], Math.min(app.rightW, vw - (app.showLeft ? leftEff : 0) - CENTER_MIN - 20)));

  onMount(() => {
    vh = window.innerHeight;
    vw = window.innerWidth;
    app.init().catch((e) => app.say(String(e), "error"));
  });

  function key(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    const typing = t?.tagName === "INPUT" || t?.tagName === "SELECT" || t?.tagName === "TEXTAREA";
    if (e.ctrlKey && e.key.toLowerCase() === "o") { e.preventDefault(); app.pickAndAdd(); }
    else if (e.ctrlKey && e.key === "Enter") { e.preventDefault(); app.startBatch(); }
    else if (!typing && e.key === "Delete" && app.tab === "current" && app.selected) app.removeSource(app.selected);
  }
</script>

<svelte:window onkeydown={key} onresize={() => { vh = window.innerHeight; vw = window.innerWidth; }} />

<div class="app">
  <header class="top">
    <div class="left">
      <button class="icon-btn" class:on={app.showLeft} aria-pressed={app.showLeft} aria-label="Show or hide the preset sidebar" title="Sidebar" onclick={() => { app.showLeft = !app.showLeft; app.saveLayout(); }}>
        <svg width="18" height="14" viewBox="0 0 18 14" aria-hidden="true"><rect x="0.5" y="0.5" width="17" height="13" rx="2.5" fill="none" stroke="currentColor"/><rect x="2" y="2" width="5" height="10" rx="1" fill="currentColor"/></svg>
      </button>
      <span class="brand">Squeeze</span>
    </div>
    <div class="seg tabs" role="tablist" aria-label="Queue views">
      {#each tabs as t}
        <button role="tab" aria-selected={app.tab === t.id} onclick={() => (app.tab = t.id)}>
          {t.label}{badge[t.id] ? ` · ${badge[t.id]}` : ""}
        </button>
      {/each}
    </div>
    <div class="right">
      <button class="icon-btn" title="Reset panel sizes" aria-label="Reset panel sizes" onclick={() => app.resetLayout()}>⟲</button>
      <button class="btn" onclick={() => app.pickAndAdd()} title="Add files (Ctrl+O)">Add files…</button>
      <button class="icon-btn" class:on={app.showRight} aria-pressed={app.showRight} aria-label="Show or hide the inspector" title="Inspector" onclick={() => { app.showRight = !app.showRight; app.saveLayout(); }}>
        <svg width="18" height="14" viewBox="0 0 18 14" aria-hidden="true"><rect x="0.5" y="0.5" width="17" height="13" rx="2.5" fill="none" stroke="currentColor"/><rect x="11" y="2" width="5" height="10" rx="1" fill="currentColor"/></svg>
      </button>
    </div>
  </header>

  <div class="mid">
    {#if app.showLeft}
      <div class="pane" style:width="{leftEff}px"><PresetTree onnew={() => (dialog = { initial: null })} onedit={(p) => (dialog = { initial: p })} /></div>
      <Splitter dir="x" label="Resize the sidebar" value={leftEff} min={LIMITS.left[0]} max={Math.min(LIMITS.left[1], Math.max(LIMITS.left[0], vw - (app.showRight ? rightEff : 0) - CENTER_MIN - 20))} onchange={(v) => (app.leftW = v)} oncommit={() => app.saveLayout()} onreset={() => { app.leftW = 250; app.saveLayout(); }} />
    {/if}

    <main class="center">
      {#if app.ffmpegMissing}
        <div class="warn" role="alert">
          FFmpeg was not found, so nothing can be converted yet. Install it with <code>winget install Gyan.FFmpeg</code>, or put
          <code>ffmpeg.exe</code> and <code>ffprobe.exe</code> in a folder named <code>ffmpeg</code> next to Squeeze, then restart.
        </div>
      {/if}
      {#if app.tab === "current"}
        <Preview height={frameH} />
        <Splitter dir="y" label="Resize the preview" value={frameH} min={LIMITS.previewMin} max={prevMax} onchange={(v) => (app.previewH = v)} oncommit={() => app.saveLayout()} onreset={() => { app.previewH = null; app.saveLayout(); }} />
        <BatchView />
      {:else if app.tab === "active"}
        <QueueView jobs={app.activeJobs} kind="active" />
      {:else}
        <QueueView jobs={app.finishedJobs} kind="completed" />
      {/if}
    </main>

    {#if app.showRight}
      <Splitter dir="x" invert label="Resize the inspector" value={rightEff} min={LIMITS.right[0]} max={Math.min(LIMITS.right[1], Math.max(LIMITS.right[0], vw - (app.showLeft ? leftEff : 0) - CENTER_MIN - 20))} onchange={(v) => (app.rightW = v)} oncommit={() => app.saveLayout()} onreset={() => { app.rightW = 310; app.saveLayout(); }} />
      <div class="pane" style:width="{rightEff}px"><Inspector /></div>
    {/if}
  </div>

  <footer class="bottom">
    <div class="b-left">
      <button class="icon-btn plus" aria-label="Add files" title="Add files (Ctrl+O)" onclick={() => app.pickAndAdd()}>＋</button>
      {#if app.overall && app.activeJobs.length}
        <div class="overall" title="Overall progress of this batch">
          <div class="obar"><div class="ofill" style="width:{app.overall.percent}%"></div></div>
          <span>Encoding {app.overall.finished}/{app.overall.total} · {Math.round(app.overall.percent)}%</span>
        </div>
      {/if}
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
      {#if app.info}
        <span class="ffv" title={app.info.ffmpeg_path ? `${app.info.ffmpeg_version ?? "ffmpeg"}
${app.info.ffmpeg_path}` : "FFmpeg not found"}>
          FFmpeg {app.info.ffmpeg_path ? (app.info.ffmpeg_version?.match(/version\s+(\S+)/)?.[1]?.split("-")[0] ?? "ok") : "missing"}
        </span>
      {/if}
      {#if app.startHint && app.tab === "current"}<span class="hint">{app.startHint}</span>{/if}
      <button class="btn primary" disabled={!app.canStart} title={app.startHint || "Start (Ctrl+Enter)"} onclick={() => app.startBatch()}>
        Start batch{app.outputCount ? ` (${app.outputCount})` : ""}
      </button>
    </div>
  </footer>
</div>

{#if app.dragging}
  <div class="dropveil" aria-hidden="true"><div>Drop files to add them to the batch</div></div>
{/if}

{#if dialog}
  <PresetDialog initial={dialog.initial} onclose={() => (dialog = null)} />
{/if}

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .top {
    flex: none; height: 40px; display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; padding: 0 10px;
    background: var(--panel); border-bottom: 1px solid #000a; box-shadow: 0 1px 0 var(--line-soft);
  }
  .left, .right { display: flex; align-items: center; gap: 10px; }
  .right { justify-content: flex-end; }
  .brand { font-weight: 600; color: var(--muted); letter-spacing: 0.02em; }
  .icon-btn.on { color: var(--accent); }
  .tabs button { min-width: 92px; }
  .mid { flex: 1; min-height: 0; display: flex; }
  .pane { flex: none; min-width: 0; overflow: hidden; background: var(--panel); }
  .center { flex: 1; min-width: 320px; display: flex; flex-direction: column; min-height: 0; overflow: hidden; background: var(--bg); border-left: 1px solid #000a; border-right: 1px solid #000a; }
  .warn { background: #4a2a12; color: #ffd8b0; padding: 6px 12px; border-bottom: 1px solid #000a; }
  .bottom {
    flex: none; height: 36px; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 0 10px;
    background: var(--panel); border-top: 1px solid #000a;
  }
  .b-left, .b-right { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .b-left { flex: 1; }
  .b-right label { display: flex; align-items: center; gap: 6px; color: var(--muted); }
  .plus { font-size: 15px; }
  .overall { display: flex; align-items: center; gap: 8px; color: var(--text); white-space: nowrap; }
  .obar { width: 140px; height: 6px; background: var(--panel-3); border-radius: 3px; overflow: hidden; }
  .ofill { height: 100%; background: var(--accent); transition: width 0.3s; }
  .msg { color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .msg.error { color: var(--danger); }
  .hint { color: var(--faint); white-space: nowrap; }
  .ffv { color: var(--faint); white-space: nowrap; }
  code { background: #0004; padding: 0 4px; border-radius: 3px; }
  .dropveil { position: fixed; inset: 0; z-index: 50; background: #0b2a5acc; border: 3px dashed var(--accent); display: grid; place-items: center; pointer-events: none; }
  .dropveil div { font-size: 18px; font-weight: 600; color: #fff; background: #0008; padding: 14px 28px; border-radius: 10px; }
</style>
