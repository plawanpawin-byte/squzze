<script lang="ts">
  import "../app.css";
  import { presetGroups, sources, inspectorFields } from "$lib/mock";

  type Tab = "Current" | "Active" | "Completed";
  const tabs: Tab[] = ["Current", "Active", "Completed"];
  let tab = $state<Tab>("Current");
  let sideTab = $state<"Presets" | "Locations">("Presets");
  let open = $state<Record<string, boolean>>({ Web: true });
  let selected = $state(0);
  let position = $state(30);
  let playing = $state(false);

  const timecode = $derived(
    "00:00:" + String(Math.round((position / 100) * 30)).padStart(2, "0") + ":00",
  );
  const locations = ["Same as source", "Desktop", "Documents", "D:\\Exports", "Choose folder…"];
</script>

<div class="app">
  <header class="top">
    <div class="brand">Squeeze</div>
    <div class="tabs" role="tablist" aria-label="Queue views">
      {#each tabs as t}
        <button role="tab" aria-selected={tab === t} class:on={tab === t} onclick={() => (tab = t)}>{t}</button>
      {/each}
    </div>
    <div class="top-actions"><button class="ghost">Add files…</button></div>
  </header>

  <aside class="side">
    <div class="side-tabs" role="tablist">
      {#each ["Presets", "Locations"] as s}
        <button role="tab" aria-selected={sideTab === s} class:on={sideTab === s} onclick={() => (sideTab = s as typeof sideTab)}>{s}</button>
      {/each}
    </div>
    {#if sideTab === "Presets"}
      <ul class="tree">
        {#each presetGroups as g}
          <li>
            <button class="group" aria-expanded={!!open[g.name]} onclick={() => (open[g.name] = !open[g.name])}>
              <span class="chev">{open[g.name] ? "▾" : "▸"}</span>{g.name}
            </button>
            {#if open[g.name] && g.items.length}
              <ul>
                {#each g.items as it}<li><button class="leaf">{it}</button></li>{/each}
              </ul>
            {/if}
          </li>
        {/each}
      </ul>
    {:else}
      <ul class="tree">
        {#each locations as l}<li><button class="leaf">{l}</button></li>{/each}
      </ul>
    {/if}
  </aside>

  <main class="center">
    <section class="preview" aria-label="Preview">
      <div class="frame"><span>{sources[selected].name}</span></div>
      <input class="scrub" type="range" min="0" max="100" bind:value={position} aria-label="Position" />
      <div class="transport">
        <button aria-label="Previous frame">⏮</button>
        <button aria-label={playing ? "Pause" : "Play"} onclick={() => (playing = !playing)}>{playing ? "⏸" : "▶"}</button>
        <button aria-label="Next frame">⏭</button>
        <span class="tc">{timecode}</span>
      </div>
    </section>

    <section class="batch" aria-label="Batch">
      {#each sources as s, i}
        <div class="source" class:sel={selected === i}>
          <button class="src-head" onclick={() => (selected = i)}>
            <span class="thumb"></span>
            <span><strong>{s.name}</strong><br /><small>{s.info}</small></span>
          </button>
          <table>
            <thead><tr><th>Preset</th><th>Location</th><th>Filename</th></tr></thead>
            <tbody>
              {#each s.outputs as o}
                <tr><td>{o.preset}</td><td>{o.location}</td><td>{o.filename}</td></tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/each}
    </section>
  </main>

  <aside class="inspector" aria-label="Job inspector">
    <h2>Job</h2>
    {#each Object.entries(inspectorFields) as [title, rows]}
      <h3>{title}</h3>
      <dl>
        {#each rows as [k, v]}<dt>{k}</dt><dd>{v}</dd>{/each}
      </dl>
    {/each}
    <h3>Timing</h3>
    <dl>
      <dt>Start</dt><dd>00:00:00:00</dd>
      <dt>End</dt><dd>{sources[selected].duration}</dd>
    </dl>
  </aside>

  <footer class="bottom">
    <label>Process on <select><option>This computer</option></select></label>
    <span class="spacer"></span>
    <span class="hw">Encoder: auto (GPU if available)</span>
    <button class="primary">Start batch</button>
  </footer>
</div>

<style>
  .app {
    display: grid;
    height: 100vh;
    grid-template-columns: 240px 1fr 280px;
    grid-template-rows: 44px 1fr 48px;
    grid-template-areas: "top top top" "side center insp" "bottom bottom bottom";
  }
  .top { grid-area: top; display: flex; align-items: center; gap: 16px; padding: 0 14px; background: var(--panel); border-bottom: 1px solid var(--line); }
  .brand { font-weight: 700; color: var(--accent); }
  .tabs { margin: 0 auto; display: flex; background: var(--panel-2); border-radius: 8px; padding: 2px; }
  .tabs button, .side-tabs button { background: none; border: 0; padding: 5px 16px; border-radius: 6px; color: var(--muted); }
  .tabs button.on, .side-tabs button.on { background: var(--bg); color: var(--text); }
  .ghost { background: none; border: 1px solid var(--line); border-radius: 6px; padding: 5px 12px; }
  .side { grid-area: side; background: var(--panel); border-right: 1px solid var(--line); overflow: auto; }
  .side-tabs { display: flex; padding: 8px; gap: 4px; border-bottom: 1px solid var(--line); }
  .tree, .tree ul { list-style: none; margin: 0; padding: 0; }
  .tree { padding: 6px; }
  .tree ul { padding-left: 16px; }
  .group, .leaf { width: 100%; text-align: left; background: none; border: 0; padding: 5px 6px; border-radius: 5px; }
  .group { font-weight: 600; }
  .leaf { color: var(--muted); }
  .group:hover, .leaf:hover { background: var(--panel-2); }
  .chev { display: inline-block; width: 14px; }
  .center { grid-area: center; display: grid; grid-template-rows: auto 1fr; min-width: 0; overflow: hidden; }
  .preview { padding: 12px; border-bottom: 1px solid var(--line); }
  .frame { height: 260px; display: grid; place-items: center; background: #000; border-radius: 8px; color: #6c7488; }
  .scrub { width: 100%; margin: 10px 0 4px; accent-color: var(--accent); }
  .transport { display: flex; align-items: center; justify-content: center; gap: 10px; }
  .transport button { background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; width: 34px; height: 28px; }
  .tc { font-variant-numeric: tabular-nums; font-size: 18px; margin-left: 16px; }
  .batch { overflow: auto; padding: 10px 12px; display: grid; gap: 10px; align-content: start; }
  .source { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; }
  .source.sel { border-color: var(--accent); }
  .src-head { display: flex; gap: 10px; align-items: center; width: 100%; text-align: left; background: none; border: 0; padding: 8px 10px; }
  .src-head small { color: var(--muted); }
  .thumb { width: 44px; height: 28px; background: var(--panel-2); border-radius: 4px; }
  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; color: var(--muted); font-weight: 500; padding: 4px 10px; border-top: 1px solid var(--line); }
  td { padding: 5px 10px; border-top: 1px solid var(--line); }
  .inspector { grid-area: insp; background: var(--panel); border-left: 1px solid var(--line); overflow: auto; padding: 10px 14px; }
  .inspector h2 { font-size: 13px; margin: 0 0 8px; }
  .inspector h3 { font-size: 12px; color: var(--muted); margin: 14px 0 4px; text-transform: uppercase; letter-spacing: 0.04em; }
  dl { display: grid; grid-template-columns: 1fr 1.3fr; gap: 4px 8px; margin: 0; }
  dt { color: var(--muted); }
  dd { margin: 0; text-align: right; }
  .bottom { grid-area: bottom; display: flex; align-items: center; gap: 12px; padding: 0 14px; background: var(--panel); border-top: 1px solid var(--line); }
  .spacer { flex: 1; }
  .hw { color: var(--muted); }
  select { background: var(--panel-2); color: var(--text); border: 1px solid var(--line); border-radius: 6px; padding: 4px 8px; margin-left: 6px; }
  .primary { background: var(--accent); color: var(--accent-ink); border: 0; border-radius: 6px; padding: 7px 18px; font-weight: 600; }
</style>
