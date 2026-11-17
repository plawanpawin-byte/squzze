<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import type { Preset } from "$lib/types";

  let { onnew, onedit }: { onnew: () => void; onedit: (p: Preset) => void } = $props();
  let side = $state<"Presets" | "Locations">("Presets");
  let open = $state<Record<string, boolean>>({ Web: true, Audio: true });

  const order = ["Web", "Social", "Archive", "Audio", "Custom"];
  const groups = $derived(
    order.map((g) => ({ name: g, items: app.presets.filter((p) => p.group === g) })),
  );
  const locations = $derived([
    { label: "Same folder as source", value: null as string | null },
    ...(app.settings?.default_dest ? [{ label: app.settings.default_dest, value: app.settings.default_dest }] : []),
  ]);
</script>

<aside class="side" aria-label="Presets and locations">
  <div class="side-tabs" role="tablist">
    {#each ["Presets", "Locations"] as s}
      <button role="tab" aria-selected={side === s} class:on={side === s} onclick={() => (side = s as typeof side)}>{s}</button>
    {/each}
  </div>

  {#if side === "Presets"}
    <label class="all"><input type="checkbox" bind:checked={app.applyAll} /> Apply to all files</label>
    <ul class="tree">
      {#each groups as g}
        <li>
          <button class="group" aria-expanded={!!open[g.name]} onclick={() => (open[g.name] = !open[g.name])}>
            <span class="chev" aria-hidden="true">{open[g.name] ? "▾" : "▸"}</span>{g.name}
          </button>
          {#if open[g.name]}
            <ul>
              {#each g.items as it (it.id)}
                <li class="row">
                  <button class="leaf" title="Add this output to the selected file" onclick={() => app.addOutput(it.id)}>{it.name}</button>
                  {#if it.group === "Custom"}
                    <button class="mini" aria-label="Edit {it.name}" onclick={() => onedit(it)}>✎</button>
                    <button class="mini" aria-label="Export {it.name}" onclick={() => app.run(() => api.exportPreset(it.id))}>⇪</button>
                    <button class="mini" aria-label="Delete {it.name}" onclick={async () => { const p = await app.run(() => api.deleteCustomPreset(it.id)); if (p) app.presets = p; }}>✕</button>
                  {/if}
                </li>
              {/each}
              {#if g.name === "Custom"}
                <li class="row"><button class="leaf act" onclick={onnew}>＋ New custom preset…</button></li>
                <li class="row"><button class="leaf act" onclick={async () => { const p = await app.run(() => api.importPreset()); if (p) app.presets = p; }}>⇩ Import preset…</button></li>
              {/if}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <ul class="tree">
      {#each locations as l}
        <li class="row">
          <button class="leaf" class:cur={app.dest === l.value} onclick={() => (app.dest = l.value)}>{app.dest === l.value ? "● " : "○ "}{l.label}</button>
        </li>
      {/each}
      {#if app.dest && !locations.some((l) => l.value === app.dest)}
        <li class="row"><button class="leaf cur">● {app.dest}</button></li>
      {/if}
      <li class="row"><button class="leaf act" onclick={() => app.chooseDest()}>Choose folder…</button></li>
    </ul>
  {/if}
</aside>

<style>
  .side { grid-area: side; background: var(--panel); border-right: 1px solid var(--line); overflow: auto; }
  .side-tabs { display: flex; padding: 8px; gap: 4px; border-bottom: 1px solid var(--line); }
  .side-tabs button { background: none; border: 0; padding: 5px 14px; border-radius: 6px; color: var(--muted); }
  .side-tabs button.on { background: var(--bg); color: var(--text); }
  .all { display: block; padding: 8px 12px 0; color: var(--muted); }
  .tree, .tree ul { list-style: none; margin: 0; padding: 0; }
  .tree { padding: 6px; }
  .tree ul { padding-left: 14px; }
  .group, .leaf { width: 100%; text-align: left; background: none; border: 0; padding: 5px 6px; border-radius: 5px; }
  .group { font-weight: 600; }
  .leaf { color: var(--muted); }
  .leaf.cur { color: var(--text); }
  .leaf.act { color: var(--accent); }
  .group:hover, .leaf:hover { background: var(--panel-2); }
  .chev { display: inline-block; width: 14px; }
  .row { display: flex; align-items: center; }
  .mini { background: none; border: 0; color: var(--muted); padding: 2px 5px; border-radius: 4px; }
  .mini:hover { background: var(--panel-2); color: var(--text); }
</style>
