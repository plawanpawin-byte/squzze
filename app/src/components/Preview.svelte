<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import { fmtTimecode } from "$lib/format";

  let pos = $state(0);
  let img = $state<string | null>(null);
  let err = $state<string | null>(null);
  let loading = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  const info = $derived(app.current?.info ?? null);
  const duration = $derived(info?.duration_secs ?? 0);
  const hasVideo = $derived(!!info?.video);

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
    }, 200);
    return () => clearTimeout(timer);
  });

  $effect(() => { app.current?.path; pos = 0; });

  function step(d: number) { pos = Math.min(Math.max(0, pos + d), Math.max(0, duration - 0.04)); }
</script>

<section class="preview" aria-label="Preview">
  <div class="frame">
    {#if img}
      <img src={img} alt="Frame of {app.current?.name} at {fmtTimecode(pos)}" />
    {:else if !app.current}
      <span>Drop files here or click “Add files…”</span>
    {:else if !hasVideo}
      <span>Audio only — no picture</span>
    {:else}
      <span>{loading ? "Loading frame…" : (err ?? "")}</span>
    {/if}
  </div>
  <input class="scrub" type="range" min="0" max={Math.max(duration, 0.1)} step="0.04" bind:value={pos} disabled={!hasVideo} aria-label="Position in the file" />
  <div class="transport">
    <button aria-label="Back 1 second" disabled={!hasVideo} onclick={() => step(-1)}>−1s</button>
    <span class="tc" aria-live="off">{fmtTimecode(pos)}</span>
    <button aria-label="Forward 1 second" disabled={!hasVideo} onclick={() => step(1)}>+1s</button>
  </div>
</section>

<style>
  .preview { padding: 12px; border-bottom: 1px solid var(--line); }
  .frame { height: 250px; display: grid; place-items: center; background: #000; border-radius: 8px; color: #6c7488; overflow: hidden; }
  .frame img { max-width: 100%; max-height: 100%; object-fit: contain; }
  .scrub { width: 100%; margin: 10px 0 4px; accent-color: var(--accent); }
  .transport { display: flex; align-items: center; justify-content: center; gap: 12px; }
  .transport button { background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; padding: 4px 10px; }
  .transport button:disabled { opacity: 0.4; cursor: default; }
  .tc { font-variant-numeric: tabular-nums; font-size: 18px; min-width: 110px; text-align: center; }
</style>
