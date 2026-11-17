<script lang="ts">
  import { api } from "$lib/api";
  import { fmtTimecode } from "$lib/format";
  import type { Job } from "$lib/types";

  let { job }: { job: Job } = $props();
  const length = $derived(job.verify?.duration_secs ?? job.duration_secs ?? 0);
  let t = $state(0);
  let wipe = $state(50);
  let src = $state<string | null>(null);
  let out = $state<string | null>(null);
  let err = $state("");
  let busy = $state(false);
  let seq = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    const at = t;
    clearTimeout(timer);
    timer = setTimeout(async () => {
      const my = ++seq;
      busy = true;
      err = "";
      try {
        const [a, b] = await Promise.all([api.preview(job.source, (job.start_secs ?? 0) + at), api.preview(job.output!, at)]);
        if (my === seq) { src = a; out = b; }
      } catch (e) {
        if (my === seq) err = String(e);
      } finally {
        if (my === seq) busy = false;
      }
    }, 180);
    return () => clearTimeout(timer);
  });
</script>

<div class="cmp">
  <div class="stage" aria-label="Compare the source with the exported file">
    {#if out}<img class="b" src={out} alt="Frame from the exported file at {fmtTimecode(t)}" />{/if}
    {#if src}<img class="a" src={src} alt="Frame from the source at {fmtTimecode((job.start_secs ?? 0) + t)}" style:clip-path="inset(0 {100 - wipe}% 0 0)" />{/if}
    {#if src && out}<div class="bar" style:left="{wipe}%"></div>{/if}
    <span class="tag l">Source</span><span class="tag r">Output</span>
    {#if !src && !out}<span class="msg">{busy ? "Loading frames…" : err || ""}</span>{/if}
  </div>
  <label>Position <input type="range" min="0" max={Math.max(length - 0.1, 0.1)} step="0.1" bind:value={t} aria-label="Time in the exported file" /> <span class="tc">{fmtTimecode(t)}</span></label>
  <label>Wipe <input type="range" min="0" max="100" bind:value={wipe} aria-label="Divider between source and output" /></label>
  <p class="note">Both pictures are decoded from the real files at the same moment{job.start_secs != null ? ` (source time = range start + position)` : ""}. Left of the line is the source, right is the exported file. Frames are previews scaled to 640 px.</p>
  {#if err && (src || out)}<p class="note bad">{err}</p>{/if}
</div>

<style>
  .cmp { display: grid; gap: 6px; }
  .stage { position: relative; background: #000; border-radius: 6px; overflow: hidden; aspect-ratio: 16 / 9; width: 100%; max-width: 604px; margin: 0 auto; display: grid; place-items: center; }
  .stage img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; }
  .bar { position: absolute; top: 0; bottom: 0; width: 2px; margin-left: -1px; background: var(--marker); box-shadow: 0 0 0 1px #0008; pointer-events: none; }
  .tag { position: absolute; top: 6px; font-size: 11px; background: #000a; color: #fff; padding: 1px 7px; border-radius: 4px; pointer-events: none; }
  .tag.l { left: 6px; } .tag.r { right: 6px; }
  .msg { color: #8a8f9c; }
  label { display: grid; grid-template-columns: 62px 1fr auto; gap: 8px; align-items: center; color: var(--muted); }
  label:nth-of-type(2) { grid-template-columns: 62px 1fr; }
  input[type="range"] { accent-color: var(--accent); width: 100%; }
  .tc { font-variant-numeric: tabular-nums; color: var(--text); }
  .note { margin: 0; color: var(--faint); font-size: 11px; line-height: 1.45; }
  .note.bad { color: var(--danger); }
</style>
