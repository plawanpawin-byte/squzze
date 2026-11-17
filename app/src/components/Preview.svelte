<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import { fmtTimecode } from "$lib/format";

  let { height }: { height: number } = $props();

  let pos = $state(0);
  let img = $state<string | null>(null);
  let err = $state<string | null>(null);
  let loading = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  const info = $derived(app.current?.info ?? null);
  const duration = $derived(info?.duration_secs ?? 0);
  const hasVideo = $derived(!!info?.video);
  const fps = $derived(info?.video?.fps ?? 30);
  const pct = $derived(duration > 0 ? (pos / duration) * 100 : 0);

  $effect(() => {
    const path = app.current?.path;
    const p = pos;
    clearTimeout(timer);
    if (!path || !hasVideo) { img = null; err = null; return; }
    timer = setTimeout(async () => {
      const my = ++seq;
      loading = true;
      try {
        const r = await api.preview(path, p);
        if (my === seq) { img = r; err = null; }
      } catch (e) {
        if (my === seq) { img = null; err = String(e); }
      } finally {
        if (my === seq) loading = false;
      }
    }, 150);
    return () => clearTimeout(timer);
  });

  $effect(() => { app.current?.path; pos = 0; });

  const inPt = $derived(app.current?.inPt ?? null);
  const outPt = $derived(app.current?.outPt ?? null);
  const rangeLeft = $derived(duration > 0 ? ((inPt ?? 0) / duration) * 100 : 0);
  const rangeWidth = $derived(duration > 0 ? (((outPt ?? duration) - (inPt ?? 0)) / duration) * 100 : 0);
  const trimmed = $derived(inPt != null || outPt != null);

  function trimKeys(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (app.tab !== "current" || e.ctrlKey || e.metaKey || e.altKey || !hasVideo) return;
    if (t?.tagName === "INPUT" && (t as HTMLInputElement).type !== "range") return;
    if (t?.tagName === "SELECT" || t?.tagName === "TEXTAREA") return;
    if (e.key === "i" || e.key === "I") { app.setIn(pos); e.preventDefault(); }
    else if (e.key === "o" || e.key === "O") { app.setOut(pos); e.preventDefault(); }
    else if (e.key === "ArrowLeft" && t?.tagName !== "INPUT") { step(e.shiftKey ? -1 : -1 / fps); e.preventDefault(); }
    else if (e.key === "ArrowRight" && t?.tagName !== "INPUT") { step(e.shiftKey ? 1 : 1 / fps); e.preventDefault(); }
  }

  function step(d: number) { pos = Math.min(Math.max(0, pos + d), Math.max(0, duration - 1 / fps)); }
</script>

<svelte:window onkeydown={trimKeys} />

<section class="viewer" aria-label="Preview">
  <div class="vbar">
    <span class="spacer"></span>
    <span class="title" title={app.current?.path ?? ""}>{app.current?.name ?? "No file selected"}</span>
    <span class="spacer right">
      {#if info?.video}<span class="chip">{info.video.width}×{info.video.height}</span>{/if}
    </span>
  </div>

  <div class="frame" style:height="{height}px">
    {#if img}
      <img src={img} alt="Frame of {app.current?.name} at {fmtTimecode(pos)}" />
    {:else if !app.current}
      <div class="drop">
        <svg width="40" height="40" viewBox="0 0 40 40" aria-hidden="true"><rect x="4" y="9" width="32" height="22" rx="3" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M20 14v12M14 20h12" stroke="currentColor" stroke-width="1.6"/></svg>
        <span>Drop video or audio files here</span>
        <button class="link" onclick={() => app.pickAndAdd()}>or choose files…</button>
      </div>
    {:else if !hasVideo}
      <span class="muted">Audio only — no picture</span>
    {:else}
      <span class="muted">{loading ? "Loading frame…" : (err ?? "")}</span>
    {/if}
  </div>

  <div class="timeline" style:--pct="{pct}%">
    {#if trimmed}<div class="range" style:left="{rangeLeft}%" style:width="{rangeWidth}%" title="Range that will be exported"></div>{/if}
    <input class="scrub" type="range" min="0" max={Math.max(duration, 0.1)} step={1 / fps} bind:value={pos} disabled={!hasVideo} aria-label="Position in the file" />
  </div>

  <div class="transport">
    <div class="tleft">
      <button class="btn small" disabled={!hasVideo} title="Set the in point at the playhead (I)" onclick={() => app.setIn(pos)}>Set In</button>
      <button class="btn small" disabled={!hasVideo} title="Set the out point at the playhead (O)" onclick={() => app.setOut(pos)}>Set Out</button>
      {#if trimmed}<button class="btn small" title="Export the whole file again" onclick={() => app.clearTrim()}>Clear</button>{/if}
    </div>
    <div class="tmid">
      <button class="icon-btn" aria-label="Back 1 second" title="Back 1 s" disabled={!hasVideo} onclick={() => step(-1)}>
        <svg width="16" height="12" viewBox="0 0 16 12" aria-hidden="true"><path d="M8 1 2 6l6 5zM15 1 9 6l6 5z" fill="currentColor"/></svg>
      </button>
      <button class="icon-btn" aria-label="Previous frame" title="Previous frame" disabled={!hasVideo} onclick={() => step(-1 / fps)}>
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M2 1v10M11 1 4 6l7 5z" fill="currentColor" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
      <span class="tc" aria-live="off">{fmtTimecode(pos)}</span>
      <button class="icon-btn" aria-label="Next frame" title="Next frame" disabled={!hasVideo} onclick={() => step(1 / fps)}>
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M10 1v10M1 1l7 5-7 5z" fill="currentColor" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
      <button class="icon-btn" aria-label="Forward 1 second" title="Forward 1 s" disabled={!hasVideo} onclick={() => step(1)}>
        <svg width="16" height="12" viewBox="0 0 16 12" aria-hidden="true"><path d="M1 1l6 5-6 5zM8 1l6 5-6 5z" fill="currentColor"/></svg>
      </button>
    </div>
    <div class="tright">
      {#if trimmed}<span class="rangeinfo" title="Only this part is exported">{fmtTimecode(inPt ?? 0)} → {outPt != null ? fmtTimecode(outPt) : "end"}</span>{/if}
      {#if duration}<span class="muted">{fmtTimecode(duration)}</span>{/if}
    </div>
  </div>
</section>

<style>
  .viewer { container-type: inline-size; flex: none; display: flex; flex-direction: column; border-bottom: 1px solid #000a; background: var(--bg); }
  .vbar { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; padding: 4px 10px; color: var(--muted); }
  .title { color: var(--text); font-weight: 500; max-width: 420px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spacer.right { display: flex; justify-content: flex-end; gap: 6px; }
  .chip { background: var(--panel-2); border: 1px solid var(--line); border-radius: 4px; padding: 0 6px; font-size: 11px; }
  .frame { margin: 0 10px; display: grid; place-items: center; background: var(--frame); overflow: hidden; }
  .frame img { max-width: 100%; max-height: 100%; object-fit: contain; display: block; }
  .drop { display: grid; justify-items: center; gap: 6px; color: var(--faint); }
  .timeline { margin: 6px 10px 0; position: relative; }
  .scrub { position: relative; z-index: 2; }
  .scrub { -webkit-appearance: none; appearance: none; width: 100%; height: 14px; background: transparent; margin: 0; }
  .scrub::-webkit-slider-runnable-track {
    height: 6px; border-radius: 3px;
    background: linear-gradient(90deg, var(--accent) 0 var(--pct), var(--panel-3) var(--pct) 100%);
  }
  .scrub::-webkit-slider-thumb { -webkit-appearance: none; width: 3px; height: 14px; margin-top: -4px; background: var(--marker); border-radius: 1px; }
  .scrub:disabled::-webkit-slider-runnable-track { background: var(--panel-2); }
  .transport { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; padding: 4px 10px 8px; }
  .tmid { display: flex; align-items: center; gap: 4px; }
  .tright { display: flex; justify-content: flex-end; align-items: center; gap: 10px; font-variant-numeric: tabular-nums; }
  .tleft { display: flex; gap: 6px; }
  .btn.small { padding: 1px 9px; font-size: 11px; white-space: nowrap; }
  .tright, .rangeinfo { white-space: nowrap; }
  @container (max-width: 640px) {
    .tright .muted, .rangeinfo { display: none; }
    .tc { min-width: 100px; font-size: 14px; margin: 0 2px; }
    .btn.small { padding: 1px 6px; }
  }
  .rangeinfo { color: var(--marker); }
  .range { position: absolute; top: 4px; height: 6px; background: var(--marker); opacity: 0.8; border-radius: 3px; pointer-events: none; z-index: 3; box-shadow: 0 0 0 1px #0006; }
  .icon-btn:disabled { opacity: 0.35; cursor: default; }
  .tc { font-variant-numeric: tabular-nums; font-size: 15px; letter-spacing: 0.04em; background: var(--panel-2); border: 1px solid var(--line); border-radius: 5px; padding: 2px 12px; margin: 0 6px; min-width: 120px; text-align: center; }
</style>
