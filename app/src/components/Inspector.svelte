<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { fmtBytes, fmtDuration, fmtTimecode } from "$lib/format";

  type Row = [string, string];
  let closed = $state<Record<string, boolean>>({});

  const info = $derived(app.tab === "current" ? (app.current?.info ?? null) : null);
  const out = $derived.by(() => {
    const s = app.current;
    const o = s?.outputs.find((x) => x.uid === app.selectedOutput) ?? s?.outputs[0];
    return o ? app.preset(o.presetId) ?? null : null;
  });
  /** Accepts "12", "1:05", "0:01:05.5". Returns seconds, or NaN when invalid. */
  function parseTime(t: string): number {
    const parts = t.trim().split(":");
    if (parts.length > 3 || parts.some((p) => !/^\d+(\.\d+)?$/.test(p))) return NaN;
    return parts.reduce((acc, p) => acc * 60 + Number(p), 0);
  }
  function commit(which: "in" | "out", e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    const txt = el.value.trim();
    const s = app.current;
    if (!s) return;
    if (txt === "") { if (which === "in") s.inPt = null; else s.outPt = null; return; }
    const v = parseTime(txt);
    if (Number.isNaN(v)) { app.say("Time format: seconds, mm:ss or hh:mm:ss.ff", "error"); el.value = fmtTimecode((which === "in" ? s.inPt : s.outPt) ?? 0); return; }
    if (which === "in") app.setIn(v); else app.setOut(v);
    el.value = fmtTimecode((which === "in" ? s.inPt : s.outPt) ?? 0);
  }
  const length = $derived.by(() => {
    const s = app.current;
    const d = s?.info?.duration_secs;
    if (!s || d == null) return null;
    return (s.outPt ?? d) - (s.inPt ?? 0);
  });
  const streams = $derived(info?.audio_streams ?? []);
  const audio = $derived(app.current?.audio ?? null);
  const chosen = $derived(streams[audio?.stream ?? 0] ?? null);
  const layout = (c: number, l: string | null) => l ?? ({ 1: "mono", 2: "stereo", 6: "5.1", 8: "7.1" } as Record<number, string>)[c] ?? `${c} ch`;
  const streamLabel = (a: (typeof streams)[number]) =>
    `#${a.index + 1} · ${a.codec} · ${layout(a.channels, a.channel_layout)}${a.sample_rate ? ` · ${(a.sample_rate / 1000).toFixed(1)} kHz` : ""}${a.language ? ` · ${a.language}` : ""}${a.title ? ` · ${a.title}` : ""}`;
  const downmixes = $derived([1, 2].filter((c) => chosen && c < chosen.channels));
  const container = (f: string) => {
    const k = f.split(",")[0];
    return ({ mov: "QuickTime movie", matroska: "Matroska", webm: "WebM", wav: "WAVE audio", flac: "FLAC audio", mp3: "MP3 audio", ogg: "Ogg", avi: "AVI" } as Record<string, string>)[k] ?? k.toUpperCase();
  };
  const summary = $derived.by(() => {
    if (!info) return [] as string[];
    const l: string[] = [container(info.format_name)];
    if (info.video) l.push(`Video: ${info.video.codec.toUpperCase()}, ${info.video.width}×${info.video.height}${info.video.fps ? `, ${info.video.fps} fps` : ""}`);
    if (info.audio) l.push(`Audio: ${info.audio.codec.toUpperCase()}, ${info.audio.channels} ch${info.audio.sample_rate ? `, ${(info.audio.sample_rate / 1000).toFixed(1)} kHz` : ""}${info.audio_streams.length > 1 ? ` (+${info.audio_streams.length - 1} more)` : ""}`);
    l.push(`Duration: ${fmtDuration(info.duration_secs)} · ${fmtBytes(info.size_bytes)}`);
    return l;
  });
  const sections = $derived.by(() => {
    const s: { title: string; rows: Row[] }[] = [];
    if (!info) return s;
    s.push({ title: "File Properties", rows: [["Container", container(info.format_name)], ["Size", fmtBytes(info.size_bytes)], ["Bit rate", info.bit_rate ? `${Math.round(info.bit_rate / 1000)} kbit/s` : "—"]] });
    if (info.video) {
      const v = info.video;
      const unk = "Unknown";
      const color = [v.color_primaries, v.color_transfer, v.color_space].filter(Boolean).join(" / ");
      s.push({
        title: "Video Properties",
        rows: [
          ["Codec", v.codec],
          ["Profile / level", [v.profile, v.level && `L${v.level}`].filter(Boolean).join(" · ") || unk],
          ["Frame size", `${v.width}×${v.height}`],
          ["Frame rate", v.fps ? `${v.fps} fps · ${v.vfr ? "variable" : "constant"}` : unk],
          ["Bit depth", v.bit_depth ? `${v.bit_depth}-bit` : unk],
          ["Pixel format", v.pix_fmt ?? unk],
          ["Field order", v.field_order ?? unk],
          ["Colour", color || unk],
          ["Range", v.color_range ?? unk],
          ["Timecode", v.timecode ?? "None"],
        ],
      });
    }
    if (info.subtitles.length) s.push({ title: "Subtitles", rows: info.subtitles.map((t, i): Row => [`#${i + 1}`, [t.codec, t.language, t.title].filter(Boolean).join(" · ")]) });
    if (out) {
      const r: Row[] = [["Preset", out.name], ["Container", `.${out.container}`]];
      if (out.video) r.push(["Video", `${out.video.codec}${out.video.rate.mode === "crf" && out.video.rate.value ? ` · quality ${out.video.rate.value}` : out.video.rate.mode === "bitrate" ? ` · ${out.video.rate.value} kbit/s` : ""}`], ["Max height", out.video.max_height ? `${out.video.max_height} px` : "keep"]);
      if (out.audio) r.push(["Audio", `${out.audio.codec}${out.audio.bitrate_kbps ? ` · ${out.audio.bitrate_kbps} kbit/s` : ""}`]);
      r.push(["Location", app.dest ?? "Source folder"]);
      s.push({ title: "Output", rows: r });
    }
    return s;
  });
</script>

<aside class="inspector" aria-label="Inspector">
  <div class="ihead">Job</div>
  <div class="ibody">
    {#if app.tab !== "current"}
      {#if app.job}
        <div class="file">
          <span class="fico" aria-hidden="true"></span>
          <div class="ftext">
            <strong>{app.job.file_name}</strong>
            <small class="path">{app.job.source}</small>
          </div>
        </div>
        <section>
          <h3>Job</h3>
          <dl>
            <dt>Status</dt><dd><span class="val">{app.job.state}</span></dd>
            <dt>Preset</dt><dd><span class="val">{app.job.preset.name}</span></dd>
            <dt>Attempt</dt><dd><span class="val">{app.job.attempt}</span></dd>
            <dt>Destination</dt><dd><span class="val">{app.job.dest_dir ?? "Source folder"}</span></dd>
            {#if app.job.output}<dt>Output</dt><dd><span class="val">{app.job.output}</span></dd>{/if}
          </dl>
        </section>
      {:else}
        <p class="none">Select a job to see its details.</p>
      {/if}
    {:else if info}
      <div class="file">
        <span class="fico" aria-hidden="true"></span>
        <div class="ftext">
          <strong>{info.file_name}</strong>
          <small class="path">{info.path}</small>
          {#each summary as l}<small>{l}</small>{/each}
        </div>
      </div>
      <section>
        <h3><button class="sh" aria-expanded={!closed.Timing} onclick={() => (closed.Timing = !closed.Timing)}>Timing<span aria-hidden="true">{closed.Timing ? "▸" : "▾"}</span></button></h3>
        {#if !closed.Timing}
          <dl>
            <dt>In</dt><dd><input class="val inp" value={app.current?.inPt != null ? fmtTimecode(app.current.inPt) : ""} placeholder="start of file" aria-label="In point" onchange={(e) => commit("in", e)} /></dd>
            <dt>Out</dt><dd><input class="val inp" value={app.current?.outPt != null ? fmtTimecode(app.current.outPt) : ""} placeholder="end of file" aria-label="Out point" onchange={(e) => commit("out", e)} /></dd>
            <dt>Length</dt><dd><span class="val">{length != null ? fmtTimecode(length) : "—"}</span></dd>
          </dl>
          <p class="tip">Move the playhead and press <kbd>I</kbd> / <kbd>O</kbd>, or type a time. Only this range is exported.</p>
        {/if}
      </section>
      {#if info.video?.hdr}
        <div class="note warn" role="note">HDR source ({info.video.color_transfer}). Squeeze does not tone-map yet, so SDR outputs will look flat and washed out.</div>
      {/if}
      {#if info.video?.vfr}
        <div class="note" role="note">Variable frame rate. Outputs keep the timing but some players handle it poorly.</div>
      {/if}
      {#if streams.length && audio}
        <section>
          <h3><button class="sh" aria-expanded={!closed.Audio} onclick={() => (closed.Audio = !closed.Audio)}>Audio ({streams.length} stream{streams.length === 1 ? "" : "s"})<span aria-hidden="true">{closed.Audio ? "▸" : "▾"}</span></button></h3>
          {#if !closed.Audio}
            <ul class="streams" role="radiogroup" aria-label="Audio stream to use">
              {#each streams as a}
                <li>
                  <label class:on={(audio.stream ?? 0) === a.index}>
                    <input type="radio" name="astream" disabled={streams.length === 1 || audio.mode === "none"} checked={(audio.stream ?? 0) === a.index}
                      onchange={() => { audio.stream = a.index; if (audio.channels && audio.channels >= a.channels) audio.channels = null; }} />
                    <span>{streamLabel(a)}</span>
                  </label>
                </li>
              {/each}
            </ul>
            <dl>
              <dt>Audio</dt>
              <dd>
                <select class="val sel" aria-label="What to do with the audio" bind:value={audio.mode} onchange={() => { if (audio.mode !== "encode") audio.channels = null; }}>
                  <option value="encode">Convert with the preset</option>
                  <option value="copy">Copy the original</option>
                  <option value="none">Remove audio</option>
                </select>
              </dd>
              {#if audio.mode === "encode"}
                <dt>Channels</dt>
                <dd>
                  <select class="val sel" aria-label="Audio channels" value={audio.channels ?? 0} onchange={(e) => (audio.channels = Number(e.currentTarget.value) || null)}>
                    <option value={0}>Keep ({chosen ? layout(chosen.channels, chosen.channel_layout) : "source"})</option>
                    {#each downmixes as c}<option value={c}>{c === 1 ? "Mono" : "Stereo"} (downmix)</option>{/each}
                  </select>
                </dd>
              {/if}
            </dl>
            {#if app.audioProblem && app.audioProblem.startsWith(app.current?.name ?? "")}<p class="tip bad">{app.audioProblem}</p>{:else if audio.mode === "copy"}<p class="tip">The audio is copied as it is: no quality loss and no re-encoding.</p>{/if}
          {/if}
        </section>
      {/if}
      {#each sections as s}
        <section>
          <h3><button class="sh" aria-expanded={!closed[s.title]} onclick={() => (closed[s.title] = !closed[s.title])}>{s.title}<span aria-hidden="true">{closed[s.title] ? "▸" : "▾"}</span></button></h3>
          {#if !closed[s.title]}
            <dl>{#each s.rows as [k, v]}<dt>{k}</dt><dd><span class="val" title={v}>{v}</span></dd>{/each}</dl>
          {/if}
        </section>
      {/each}
    {:else}
      <p class="none">Select a file to see its properties.</p>
    {/if}
  </div>
</aside>

<style>
  .inspector { height: 100%; display: flex; flex-direction: column; background: var(--panel); }
  .ihead { padding: 9px 12px; font-weight: 600; border-bottom: 1px solid var(--line-soft); }
  .ibody { flex: 1; min-height: 0; overflow: auto; }
  .file { display: flex; gap: 10px; padding: 10px 12px; border-bottom: 1px solid var(--line-soft); }
  .fico { width: 26px; height: 32px; flex: none; border-radius: 3px; background: linear-gradient(160deg, #3a3a3c, #2a2a2c); border: 1px solid var(--line); position: relative; }
  .fico::after { content: ""; position: absolute; left: 7px; top: 10px; border-left: 9px solid var(--muted); border-top: 6px solid transparent; border-bottom: 6px solid transparent; }
  .ftext { min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .ftext strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ftext small { color: var(--muted); font-size: 11px; line-height: 1.45; }
  .path { word-break: break-all; color: var(--faint) !important; }
  section { border-bottom: 1px solid var(--line-soft); padding: 2px 0 8px; }
  h3 { margin: 0; }
  .sh { width: 100%; display: flex; justify-content: space-between; align-items: center; background: none; border: 0; padding: 7px 12px 5px; font-weight: 600; font-size: 12px; color: var(--text); }
  section > h3:not(:has(.sh)) { padding: 7px 12px 5px; font-size: 12px; }
  .sh span { color: var(--faint); font-size: 10px; }
  dl { display: grid; grid-template-columns: 92px 1fr; gap: 4px 8px; margin: 0; padding: 0 12px; align-items: center; }
  dt { color: var(--muted); text-align: right; }
  dd { margin: 0; min-width: 0; }
  .val { display: block; background: var(--panel-2); border: 1px solid var(--line); border-radius: 4px; padding: 1px 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-variant-numeric: tabular-nums; }
  .none { color: var(--faint); padding: 12px; }
  .note { margin: 8px 12px 0; padding: 6px 9px; border-radius: 6px; background: var(--panel-2); color: var(--muted); font-size: 11px; line-height: 1.45; }
  .note.warn { background: #f5a62318; color: var(--warn); }
  .streams { list-style: none; margin: 0 0 6px; padding: 0 12px; display: grid; gap: 3px; }
  .streams label { display: flex; gap: 7px; align-items: flex-start; padding: 3px 6px; border-radius: 5px; color: var(--muted); }
  .streams label.on { background: var(--sel); color: var(--text); }
  .streams input { margin-top: 2px; }
  .sel { width: 100%; font: inherit; color: var(--text); }
  .tip.bad { color: var(--danger); }
  .inp { width: 100%; font: inherit; color: var(--text); }
  .inp::placeholder { color: var(--faint); }
  .tip { margin: 6px 12px 0; color: var(--faint); font-size: 11px; line-height: 1.4; }
  kbd { background: var(--panel-3); border: 1px solid var(--line); border-radius: 3px; padding: 0 4px; font: inherit; font-size: 10.5px; }
</style>
