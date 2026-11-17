<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import type { Preset } from "$lib/types";

  let { onnew, onedit }: { onnew: () => void; onedit: (p: Preset) => void } = $props();
  let side = $state<"Presets" | "Locations">("Presets");
  let open = $state<Record<string, boolean>>({ Web: true, Audio: false, Social: false, Archive: false });
  let picked = $state<string | null>(null);

  const builtinGroups = ["Web", "Social", "Archive", "Audio"];
  const q = $derived(app.presetFilter.trim().toLowerCase());
  const match = (p: Preset) => !q || p.name.toLowerCase().includes(q) || p.container.includes(q);
  const groups = $derived(builtinGroups.map((g) => ({ name: g, items: app.presets.filter((p) => p.group === g && match(p)) })).filter((g) => !q || g.items.length));
  const custom = $derived(app.presets.filter((p) => p.group === "Custom" && match(p)));

  const exampleName = $derived(
    ((app.settings?.template || "{name}-{preset}").replace("{name}", "clip").replace("{preset}", "h264-1080")).replace(/[\\/:*?"<>|]/g, "_") + ".mp4",
  );

  const favs = $derived(app.settings?.favorites ?? []);
  const favorites = $derived(favs.map((id) => app.presets.find((p) => p.id === id)).filter((p): p is Preset => !!p && match(p)));
  const isFav = (id: string) => favs.includes(id);
  function toggleFav(id: string) {
    app.setSetting({ favorites: isFav(id) ? favs.filter((x) => x !== id) : [...favs, id] });
  }
  async function duplicate(id: string) {
    const p = await app.run(() => api.duplicatePreset(id));
    if (p) {
      app.presets = p;
      open.Custom = true;
      app.say("Copied to Custom — edit it with ✎");
    }
  }

  function add(id: string) {
    picked = id;
    app.addOutput(id);
  }
  const isOpen = (g: string) => !!q || !!open[g];
</script>

<aside class="side" aria-label="Presets and locations">
  <div class="head">
    <div class="seg" role="tablist">
      {#each ["Presets", "Locations"] as s}
        <button role="tab" aria-selected={side === s} onclick={() => (side = s as typeof side)}>{s}</button>
      {/each}
    </div>
  </div>

  <div class="body">
    {#if side === "Presets"}
      {#if favorites.length}
        <div class="sect">Favourites</div>
        <ul class="tree">
          {#each favorites as it (it.id)}
            <li class="row" class:picked={picked === it.id}>
              <button class="leaf" class:off={!!app.unavailable[it.id]} title={app.unavailable[it.id] ?? "Click to add this output to the selected file"} onclick={() => add(it.id)}>
                <span class="ico {it.video ? 'film' : 'wave'}" aria-hidden="true"></span>{it.name}
              </button>
              <button class="mini on" aria-label="Remove {it.name} from favourites" title="Unstar" onclick={() => toggleFav(it.id)}>★</button>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="sect">Built-in</div>
      <ul class="tree">
        {#each groups as g}
          <li>
            <button class="group" aria-expanded={isOpen(g.name)} onclick={() => (open[g.name] = !open[g.name])}>
              <span class="chev" aria-hidden="true">{isOpen(g.name) ? "▾" : "▸"}</span>
              <span class="ico folder" aria-hidden="true"></span>{g.name}
            </button>
            {#if isOpen(g.name)}
              <ul>
                {#each g.items as it (it.id)}
                  <li class="row" class:picked={picked === it.id}>
                    <button class="leaf" class:off={!!app.unavailable[it.id]} title={app.unavailable[it.id] ?? `${it.name} — click to add this output to the selected file`} aria-disabled={!!app.unavailable[it.id]} onclick={() => add(it.id)}>
                      <span class="ico {it.video ? 'film' : 'wave'}" aria-hidden="true"></span>{it.name}{#if app.unavailable[it.id]}<span class="na"> · not available</span>{/if}
                    </button>
                    <button class="mini hov" class:on={isFav(it.id)} aria-label="{isFav(it.id) ? 'Remove' : 'Add'} {it.name} {isFav(it.id) ? 'from' : 'to'} favourites" title="Favourite" onclick={() => toggleFav(it.id)}>{isFav(it.id) ? "★" : "☆"}</button>
                    <button class="mini hov" aria-label="Duplicate {it.name}" title="Duplicate to Custom" onclick={() => duplicate(it.id)}>⧉</button>
                  </li>
                {/each}
              </ul>
            {/if}
          </li>
        {/each}
      </ul>
      <div class="sect">Custom</div>
      <ul class="tree">
        {#each custom as it (it.id)}
          <li class="row" class:picked={picked === it.id}>
            <button class="leaf" onclick={() => add(it.id)}><span class="ico {it.video ? 'film' : 'wave'}" aria-hidden="true"></span>{it.name}</button>
            <button class="mini hov" class:on={isFav(it.id)} aria-label="{isFav(it.id) ? 'Remove' : 'Add'} {it.name} {isFav(it.id) ? 'from' : 'to'} favourites" title="Favourite" onclick={() => toggleFav(it.id)}>{isFav(it.id) ? "★" : "☆"}</button>
            <button class="mini" aria-label="Edit {it.name}" title="Edit" onclick={() => onedit(it)}>✎</button>
            <button class="mini" aria-label="Duplicate {it.name}" title="Duplicate" onclick={() => duplicate(it.id)}>⧉</button>
            <button class="mini" aria-label="Export {it.name}" title="Export" onclick={() => app.run(() => api.exportPreset(it.id))}>⇪</button>
            <button class="mini" aria-label="Delete {it.name}" title="Delete" onclick={async () => { const p = await app.run(() => api.deleteCustomPreset(it.id)); if (p) app.presets = p; }}>✕</button>
          </li>
        {:else}
          <li class="empty">No custom presets</li>
        {/each}
      </ul>
    {:else}
      <div class="sect">Destination</div>
      <ul class="tree">
        <li class="row" class:picked={app.dest === null}>
          <button class="leaf" onclick={() => (app.dest = null)}><span class="ico folder" aria-hidden="true"></span>Source (same folder)</button>
        </li>
        {#if app.settings?.default_dest}
          <li class="row" class:picked={app.dest === app.settings.default_dest}>
            <button class="leaf" onclick={() => (app.dest = app.settings!.default_dest)}><span class="ico folder" aria-hidden="true"></span>{app.settings.default_dest}</button>
          </li>
        {/if}
        {#if app.dest && app.dest !== app.settings?.default_dest}
          <li class="row picked"><button class="leaf" title={app.dest}><span class="ico folder" aria-hidden="true"></span>{app.dest}</button></li>
        {/if}
        <li class="row"><button class="leaf act" onclick={() => app.chooseDest()}>Choose folder…</button></li>
      </ul>
      <p class="hint">Outputs of the next batch are saved here.</p>
      {#if app.settings}
        <div class="sect">File names</div>
        <div class="form">
          <label>Name template
            <input class="field" value={app.settings.template} maxlength="120" aria-label="Output name template"
              onchange={(e) => app.setSetting({ template: e.currentTarget.value })} />
          </label>
          <p class="hint tight">Use <code>{"{name}"}</code> for the source name and <code>{"{preset}"}</code> for the preset id. Example: <code>{exampleName}</code></p>
          <label>If the file already exists
            <select class="field" value={app.settings.overwrite} aria-label="If the output file already exists"
              onchange={(e) => app.setSetting({ overwrite: e.currentTarget.value as "rename" | "replace" | "skip" })}>
              <option value="rename">Keep both (add a number)</option>
              <option value="skip">Skip — fail the job</option>
              <option value="replace">Replace the old file</option>
            </select>
          </label>
          <label class="chk"><input type="checkbox" checked={app.settings.strip_metadata} onchange={(e) => app.setSetting({ strip_metadata: e.currentTarget.checked })} aria-label="Remove metadata from outputs" /> Remove metadata (title, location, camera info)</label>
          <p class="hint tight">Off by default: tags from the source are copied to the output.</p>
          {#if app.settings.overwrite === "replace"}<p class="hint tight warnc">Existing files with the same name will be overwritten. The source file is never overwritten.</p>{/if}
        </div>
      {/if}
    {/if}
  </div>

  <div class="foot">
    {#if side === "Presets"}
      <button class="icon-btn" aria-label="New custom preset" title="New custom preset" onclick={onnew}>＋</button>
      <button class="icon-btn" aria-label="Import preset" title="Import preset" onclick={async () => { const p = await app.run(() => api.importPreset()); if (p) app.presets = p; }}>⇩</button>
      <input class="field search" type="search" placeholder="Search" aria-label="Search presets" bind:value={app.presetFilter} />
    {/if}
  </div>
</aside>

<style>
  .side { height: 100%; display: flex; flex-direction: column; background: var(--panel); }
  .head { padding: 8px 10px; display: flex; justify-content: center; }
  .head .seg button { padding: 2px 16px; }
  .body { flex: 1; min-height: 0; overflow: auto; padding: 2px 6px 8px; }
  .sect { font-size: 10.5px; font-weight: 700; color: var(--faint); text-transform: uppercase; letter-spacing: 0.06em; padding: 8px 6px 3px; }
  .tree, .tree ul { list-style: none; margin: 0; padding: 0; }
  .tree ul { padding-left: 14px; }
  .group, .leaf { flex: 1; min-width: 0; width: 100%; display: flex; align-items: center; gap: 6px; text-align: left; background: none; border: 0; padding: 3px 6px; border-radius: 5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .group { color: var(--text); }
  .leaf { color: var(--text); opacity: 0.9; }
  .leaf.act { color: var(--accent); }
  .leaf.off { opacity: 0.45; }
  .na { font-size: 10.5px; color: var(--warn); }
  .group:hover, .row:hover { background: var(--panel-3); }
  .row { display: flex; align-items: center; border-radius: 5px; }
  .row.picked { background: var(--sel); }
  .chev { width: 10px; color: var(--muted); font-size: 10px; }
  .ico { width: 14px; height: 11px; flex: none; border-radius: 2px; display: inline-block; }
  .ico.folder { background: linear-gradient(#5b9bf0, #3d7fd9); border-radius: 2px 4px 2px 2px; }
  .ico.film { border: 1.5px solid var(--muted); position: relative; }
  .ico.film::after { content: ""; position: absolute; inset: 2px 3px; border-left: 1.5px solid var(--muted); border-right: 1.5px solid var(--muted); }
  .ico.wave { background: repeating-linear-gradient(90deg, var(--muted) 0 1.5px, transparent 1.5px 3.5px); mask: linear-gradient(transparent 20%, #000 20% 80%, transparent 80%); }
  .mini { background: none; border: 0; color: var(--muted); padding: 2px 4px; border-radius: 4px; }
  .mini:hover { color: var(--text); background: var(--panel-2); }
  .mini.on { color: var(--marker); }
  .row .mini.hov { visibility: hidden; }
  .row:hover .mini.hov, .row .mini.hov.on, .row:focus-within .mini.hov { visibility: visible; }
  .empty, .hint { color: var(--faint); padding: 3px 8px; list-style: none; }
  .foot { display: flex; align-items: center; gap: 4px; padding: 6px 8px; border-top: 1px solid var(--line-soft); }
  .search { flex: 1; min-width: 0; }
  .form { display: grid; gap: 8px; padding: 2px 8px 10px; }
  .form label { display: grid; gap: 3px; color: var(--muted); }
  .form .field { width: 100%; }
  .form .chk { display: flex; gap: 6px; align-items: flex-start; }
  .tight { padding: 0; margin: -2px 0 0; font-size: 11px; line-height: 1.4; }
  .warnc { color: var(--warn); }
  code { background: var(--panel-3); border-radius: 3px; padding: 0 3px; }
</style>
