<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { stem, fmtDuration } from "$lib/format";

  function fileFor(name: string, presetId: string): string {
    const p = app.preset(presetId);
    return p ? `${stem(name)}-${presetId}.${p.container}` : "";
  }
  const where = $derived(app.dest ? app.dest.split(/[\\/]/).pop() || app.dest : "Source");
  const names = $derived(app.sources.map((s) => s.name).join(", "));

  $effect(() => {
    for (const s of app.sources) {
      if (s.info?.video) app.loadThumb(s.path, Math.min(1, (s.info.duration_secs ?? 0) / 2));
    }
  });
</script>

<section class="batch" aria-label="Batch">
  <div class="bhead">
    <span class="bt">Batch</span>
    <span class="names" title={names}>{names || "Empty"}</span>
    <label class="all"><input type="checkbox" bind:checked={app.applyAll} /> Apply presets to all files</label>
    <button class="link" onclick={() => app.pickAndAdd()}>Add</button>
  </div>

  <div class="list">
    {#if app.sources.length === 0}
      <p class="empty">No files yet. Drag files into the window or use <strong>Add files…</strong> (Ctrl+O).<br />Then click presets in the sidebar to add outputs.</p>
    {/if}
    {#each app.sources as s (s.path)}
      <div class="source" class:sel={app.selected === s.path}>
        <div class="src-head">
          <button class="pick" onclick={() => (app.selected = s.path)} aria-pressed={app.selected === s.path}>
            <span class="thumb">
              {#if app.thumbs[s.path]}<img src={app.thumbs[s.path]} alt="" />{:else if s.info && !s.info.video}<span class="aud" aria-hidden="true">♪</span>{/if}
            </span>
            <span class="meta">
              <strong>{s.name}</strong>
              {#if s.info && (s.inPt != null || s.outPt != null)}<small class="mark">Range {fmtDuration(s.inPt ?? 0)} – {s.outPt != null ? fmtDuration(s.outPt) : "end"}</small>{/if}
              {#if s.info}
                <small>
                  {s.info.video ? `${s.info.video.codec} · ${s.info.video.width}×${s.info.video.height}${s.info.video.fps ? ` · ${s.info.video.fps} fps` : ""}` : "audio"}{s.info.audio ? ` · ${s.info.audio.codec} ${s.info.audio.channels} ch` : ""} · {fmtDuration(s.info.duration_secs)}
                </small>
              {:else}
                <small class="bad">Cannot read this file: {s.error}</small>
              {/if}
            </span>
          </button>
          {#if s.info}<button class="link add" onclick={() => app.addDefaultOutput(s.path)}>Add</button>{/if}
          <button class="icon-btn" aria-label="Remove {s.name}" title="Remove file" onclick={() => app.removeSource(s.path)}>✕</button>
        </div>
        {#if s.info}
          <table>
            <thead><tr><th class="c1">Preset</th><th class="c2">Location</th><th>Filename</th><th class="c4"></th></tr></thead>
            <tbody>
              {#each s.outputs as o (o.uid)}
                <tr class:rsel={app.selectedOutput === o.uid} onclick={() => { app.selected = s.path; app.selectedOutput = o.uid; }}>
                  <td class="c1"><div class="cell">
                    <span class="ico {app.preset(o.presetId)?.video ? 'film' : 'wave'}" aria-hidden="true"></span>
                    <select aria-label="Preset for {s.name}" bind:value={o.presetId}>
                      {#each app.presets as p (p.id)}
                        <option value={p.id} disabled={(!!p.video && !s.info.video) || !!app.unavailable[p.id]} title={app.unavailable[p.id]}>{p.name}{app.unavailable[p.id] ? " (not available)" : ""}</option>
                      {/each}
                    </select>
                  </div></td>
                  <td class="c2"><span class="ico folder" aria-hidden="true"></span><span class="ell" title={app.dest ?? "Same folder as the source"}>{where}</span></td>
                  <td class="ell" title={fileFor(s.name, o.presetId)}>{fileFor(s.name, o.presetId)}</td>
                  <td class="c4"><button class="icon-btn" aria-label="Remove this output" title="Remove output" onclick={(e) => { e.stopPropagation(); app.removeOutput(s.path, o.uid); }}>✕</button></td>
                </tr>
              {:else}
                <tr><td colspan="4" class="hint">No outputs — click a preset in the sidebar or “Add”.</td></tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {/each}
  </div>
</section>

<style>
  .batch { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .bhead { display: flex; align-items: center; gap: 10px; padding: 5px 12px; background: var(--panel); border-bottom: 1px solid var(--line-soft); }
  .bt { font-weight: 600; }
  .names { flex: 1; min-width: 0; color: var(--muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .all { display: flex; align-items: center; gap: 5px; color: var(--muted); white-space: nowrap; }
  .list { flex: 1; min-height: 0; overflow: auto; padding: 8px 10px; display: flex; flex-direction: column; gap: 8px; }
  .empty { color: var(--faint); padding: 28px; text-align: center; line-height: 1.7; }
  .source { background: var(--panel); border: 1px solid var(--line-soft); border-radius: 7px; overflow: hidden; }
  .source.sel { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent-soft); }
  .src-head { display: flex; align-items: center; gap: 8px; padding-right: 6px; }
  .pick { flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px; text-align: left; background: none; border: 0; padding: 6px 10px; }
  .thumb { width: 56px; height: 32px; flex: none; background: #000; border-radius: 3px; overflow: hidden; display: grid; place-items: center; }
  .thumb img { width: 100%; height: 100%; object-fit: cover; }
  .aud { color: var(--muted); font-size: 16px; }
  .meta { min-width: 0; display: flex; flex-direction: column; }
  .meta strong { font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .meta small { color: var(--muted); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bad { color: var(--danger) !important; white-space: normal !important; }
  .mark { color: var(--marker) !important; }
  .add { margin-right: 6px; }
  table { width: 100%; border-collapse: collapse; table-layout: fixed; }
  th { text-align: left; color: var(--faint); font-weight: 500; font-size: 11px; padding: 3px 10px; border-top: 1px solid var(--line-soft); background: var(--bg); }
  td { padding: 2px 10px; border-top: 1px solid var(--line-soft); height: 26px; }
  tbody tr { cursor: default; }
  tbody tr:hover { background: var(--panel-3); }
  tr.rsel { background: var(--sel) !important; }
  .c1 { width: 40%; }
  .c2 { width: 22%; }
  .c4 { width: 34px; padding: 0; text-align: center; }
  .c1, .c2 { white-space: nowrap; }
  .cell { display: flex; align-items: center; gap: 6px; min-width: 0; }
  .c2 { overflow: hidden; }
  .ell { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  td.c2 .ell { display: inline-block; max-width: calc(100% - 20px); vertical-align: middle; margin-left: 6px; }
  .hint { color: var(--faint); }
  select { flex: 1; min-width: 0; background: transparent; color: var(--text); border: 0; padding: 2px 0; }
  select:hover { background: var(--panel-2); }
  .ico { width: 14px; height: 11px; flex: none; display: inline-block; vertical-align: middle; border-radius: 2px; }
  .ico.folder { background: linear-gradient(#5b9bf0, #3d7fd9); border-radius: 2px 4px 2px 2px; }
  .ico.film { border: 1.5px solid var(--muted); position: relative; }
  .ico.film::after { content: ""; position: absolute; inset: 2px 3px; border-left: 1.5px solid var(--muted); border-right: 1.5px solid var(--muted); }
  .ico.wave { background: repeating-linear-gradient(90deg, var(--muted) 0 1.5px, transparent 1.5px 3.5px); }
</style>
