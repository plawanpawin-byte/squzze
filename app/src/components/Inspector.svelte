<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { fmtBytes, fmtDuration } from "$lib/format";

  type Row = [string, string];
  const sections = $derived.by(() => {
    const i = app.current?.info;
    if (!i) return [] as { title: string; rows: Row[] }[];
    const out: { title: string; rows: Row[] }[] = [
      { title: "File", rows: [["Name", i.file_name], ["Container", i.format_name.split(",")[0]], ["Size", fmtBytes(i.size_bytes)], ["Duration", fmtDuration(i.duration_secs)], ["Bit rate", i.bit_rate ? `${Math.round(i.bit_rate / 1000)} kbit/s` : "—"]] },
    ];
    if (i.video) out.push({ title: "Video", rows: [["Codec", i.video.codec], ["Resolution", `${i.video.width}×${i.video.height}`], ["Frame rate", i.video.fps ? `${i.video.fps} fps` : "—"], ["Pixel format", i.video.pix_fmt ?? "—"], ["Field order", i.video.field_order ?? "—"]] });
    if (i.audio) out.push({ title: "Audio", rows: [["Codec", i.audio.codec], ["Channels", String(i.audio.channels)], ["Sample rate", i.audio.sample_rate ? `${i.audio.sample_rate} Hz` : "—"]] });
    return out;
  });
</script>

<aside class="inspector" aria-label="Job inspector">
  <h2>{app.tab === "current" ? "Source" : "Job"}</h2>
  {#if app.tab !== "current" && app.job}
    <h3>Job</h3>
    <dl>
      <dt>Status</dt><dd>{app.job.state}</dd>
      <dt>Preset</dt><dd>{app.job.preset.name}</dd>
      <dt>Container</dt><dd>.{app.job.preset.container}</dd>
      <dt>Attempt</dt><dd>{app.job.attempt}</dd>
      <dt>Destination</dt><dd>{app.job.dest_dir ?? "Same as source"}</dd>
    </dl>
  {:else if sections.length === 0}
    <p class="muted">Select a file to see its properties.</p>
  {/if}
  {#if app.tab === "current"}
    {#each sections as s}
      <h3>{s.title}</h3>
      <dl>{#each s.rows as [k, v]}<dt>{k}</dt><dd>{v}</dd>{/each}</dl>
    {/each}
  {/if}
</aside>

<style>
  .inspector { grid-area: insp; background: var(--panel); border-left: 1px solid var(--line); overflow: auto; padding: 10px 14px; }
  h2 { font-size: 13px; margin: 0 0 8px; }
  h3 { font-size: 12px; color: var(--muted); margin: 14px 0 4px; text-transform: uppercase; letter-spacing: 0.04em; }
  dl { display: grid; grid-template-columns: 1fr 1.4fr; gap: 4px 8px; margin: 0; }
  dt { color: var(--muted); }
  dd { margin: 0; text-align: right; word-break: break-all; }
  .muted { color: var(--muted); }
</style>
