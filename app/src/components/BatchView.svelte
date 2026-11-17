<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { stem, fmtDuration } from "$lib/format";

  function fileFor(name: string, presetId: string): string {
    const p = app.preset(presetId);
    return p ? `${stem(name)}-${presetId}.${p.container}` : "";
  }
  const where = $derived(app.dest ?? "Same as source");
</script>

<section class="batch" aria-label="Batch">
  {#if app.sources.length === 0}
    <p class="empty">No files yet. Drag video or audio files into this window, or use <strong>Add files…</strong> (Ctrl+O).</p>
  {/if}
  {#each app.sources as s (s.path)}
    <div class="source" class:sel={app.selected === s.path}>
      <div class="src-head">
        <button class="pick" onclick={() => (app.selected = s.path)} aria-pressed={app.selected === s.path}>
          <strong>{s.name}</strong><br />
          {#if s.info}
            <small>
              {s.info.video ? `${s.info.video.codec} ${s.info.video.width}×${s.info.video.height}${s.info.video.fps ? ` · ${s.info.video.fps} fps` : ""}` : "audio"}
              {s.info.audio ? ` · ${s.info.audio.codec} ${s.info.audio.channels} ch` : ""} · {fmtDuration(s.info.duration_secs)}
            </small>
          {:else}
            <small class="bad">Cannot read this file: {s.error}</small>
          {/if}
        </button>
        <button class="x" aria-label="Remove {s.name}" onclick={() => app.removeSource(s.path)}>✕</button>
      </div>
      {#if s.info}
        <table>
          <thead><tr><th>Preset</th><th>Location</th><th>Output file</th><th></th></tr></thead>
          <tbody>
            {#each s.outputs as o (o.uid)}
              <tr>
                <td>
                  <select aria-label="Preset for {s.name}" bind:value={o.presetId}>
                    {#each app.presets as p (p.id)}
                      <option value={p.id} disabled={!!p.video && !s.info.video}>{p.name}</option>
                    {/each}
                  </select>
                </td>
                <td class="loc" title={where}>{where}</td>
                <td class="fn">{fileFor(s.name, o.presetId)}</td>
                <td><button class="x" aria-label="Remove this output" onclick={() => app.removeOutput(s.path, o.uid)}>✕</button></td>
              </tr>
            {:else}
              <tr><td colspan="4" class="hint">No outputs. Pick a preset on the left.</td></tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/each}
</section>

<style>
  .batch { overflow: auto; padding: 10px 12px; display: grid; gap: 10px; align-content: start; }
  .empty { color: var(--muted); padding: 24px; text-align: center; }
  .source { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; }
  .source.sel { border-color: var(--accent); }
  .src-head { display: flex; align-items: center; }
  .pick { flex: 1; text-align: left; background: none; border: 0; padding: 8px 10px; }
  .pick small { color: var(--muted); }
  .bad { color: var(--danger) !important; }
  .x { background: none; border: 0; color: var(--muted); padding: 6px 10px; }
  .x:hover { color: var(--text); }
  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; color: var(--muted); font-weight: 500; padding: 4px 10px; border-top: 1px solid var(--line); }
  td { padding: 4px 10px; border-top: 1px solid var(--line); }
  .loc, .fn { max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .hint { color: var(--muted); }
  select { background: var(--panel-2); color: var(--text); border: 1px solid var(--line); border-radius: 6px; padding: 3px 6px; max-width: 240px; }
</style>
