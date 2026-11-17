<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import type { Preset } from "$lib/types";

  let { initial, onclose }: { initial: Preset | null; onclose: () => void } = $props();
  const seed = () => initial; // form fields start from the preset being edited

  const vcodecs = [["h264", "H.264"], ["hevc", "HEVC"], ["av1", "AV1"], ["vp9", "VP9"], ["prores422", "ProRes 422"], ["prores4444", "ProRes 4444"], ["ffv1", "FFV1 (lossless)"]];
  const acodecs = [["aac", "AAC"], ["opus", "Opus"], ["mp3", "MP3"], ["flac", "FLAC"], ["pcm24", "PCM 24-bit"]];
  const vcontainers = ["mp4", "mov", "mkv", "webm"];
  const acontainers = ["m4a", "opus", "flac", "wav", "mp3"];

  let name = $state(seed()?.name ?? "");
  let mode = $state<"video" | "audio">(seed() && !seed()!.video ? "audio" : "video");
  let container = $state(seed()?.container ?? "mp4");
  let vcodec = $state(seed()?.video?.codec ?? "h264");
  let rate = $state<"crf" | "bitrate">(seed()?.video?.rate.mode ?? "crf");
  let rateValue = $state(seed()?.video?.rate.value ?? 22);
  let height = $state<number | null>(seed()?.video?.max_height ?? null);
  let fps = $state<number | null>(seed()?.video?.fps ?? null);
  let acodec = $state(seed()?.audio?.codec ?? "aac");
  let abitrate = $state<number | null>(seed()?.audio?.bitrate_kbps ?? 192);
  let channels = $state<number | null>(seed()?.audio?.channels ?? null);
  let error = $state("");

  const id = $derived(initial?.id ?? ("custom-" + (name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40) || "preset")));

  $effect(() => {
    const list = mode === "video" ? vcontainers : acontainers;
    if (!list.includes(container)) container = list[0];
  });

  async function save() {
    error = "";
    const preset: Preset = {
      id, name: name.trim(), group: "Custom", container,
      video: mode === "video" ? { codec: vcodec, rate: { mode: rate, value: Number(rateValue) }, max_height: height ? Number(height) : null, fps: fps ? Number(fps) : null } : null,
      audio: { codec: acodec, bitrate_kbps: abitrate ? Number(abitrate) : null, channels: channels ? Number(channels) : null },
    };
    if (mode === "video" && ["prores422", "prores4444", "ffv1"].includes(vcodec)) preset.video!.rate = { mode: "crf", value: 0 };
    try {
      app.presets = await api.saveCustomPreset(preset);
      onclose();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />
<div class="backdrop" role="presentation" onclick={onclose}></div>
<div class="dlg" role="dialog" aria-modal="true" aria-label="Custom preset">
  <h2>{initial ? "Edit" : "New"} custom preset</h2>
  <label>Name <input bind:value={name} maxlength="80" /></label>
  <label>Type
    <select bind:value={mode}><option value="video">Video + audio</option><option value="audio">Audio only</option></select>
  </label>
  <label>Container
    <select bind:value={container}>{#each mode === "video" ? vcontainers : acontainers as c}<option value={c}>.{c}</option>{/each}</select>
  </label>
  {#if mode === "video"}
    <label>Video codec <select bind:value={vcodec}>{#each vcodecs as [v, l]}<option value={v}>{l}</option>{/each}</select></label>
    {#if !["prores422", "prores4444", "ffv1"].includes(vcodec)}
      <label>Rate control
        <span class="inline">
          <select bind:value={rate}><option value="crf">Quality (CRF)</option><option value="bitrate">Bitrate (kbit/s)</option></select>
          <input type="number" min="0" max="500000" bind:value={rateValue} aria-label="Rate value" />
        </span>
      </label>
    {/if}
    <label>Max height (px, empty = keep) <input type="number" min="144" max="8640" bind:value={height} /></label>
    <label>Frame rate (empty = keep) <input type="number" min="1" max="240" step="0.001" bind:value={fps} /></label>
  {/if}
  <label>Audio codec <select bind:value={acodec}>{#each acodecs as [v, l]}<option value={v}>{l}</option>{/each}</select></label>
  <label>Audio bitrate kbit/s <input type="number" min="16" max="1536" bind:value={abitrate} /></label>
  <label>Channels (empty = keep) <input type="number" min="1" max="8" bind:value={channels} /></label>
  {#if error}<p class="err" role="alert">{error}</p>{/if}
  <div class="acts">
    <button onclick={onclose}>Cancel</button>
    <button class="primary" disabled={!name.trim()} onclick={save}>Save</button>
  </div>
</div>

<style>
  .backdrop { position: fixed; inset: 0; background: #0009; z-index: 10; }
  .dlg { position: fixed; z-index: 11; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 420px; max-height: 90vh; overflow: auto; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; padding: 16px 18px; display: grid; gap: 10px; }
  h2 { margin: 0; font-size: 15px; }
  label { display: grid; gap: 4px; color: var(--muted); }
  input, select { background: var(--panel-2); color: var(--text); border: 1px solid var(--line); border-radius: 6px; padding: 5px 8px; font: inherit; }
  .inline { display: flex; gap: 8px; }
  .inline input { width: 110px; }
  .err { color: var(--danger); margin: 0; }
  .acts { display: flex; justify-content: flex-end; gap: 8px; }
  .acts button { background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; padding: 6px 16px; }
  .acts .primary { background: var(--accent); color: var(--accent-ink); border: 0; font-weight: 600; }
  .acts .primary:disabled { opacity: 0.4; }
</style>
