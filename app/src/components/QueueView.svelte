<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import { fmtBytes, fmtDuration, fmtEta } from "$lib/format";
  import Compare from "./Compare.svelte";
  import type { Job } from "$lib/types";

  let { jobs, kind }: { jobs: Job[]; kind: "active" | "completed" } = $props();
  let open = $state<Record<number, boolean>>({});
  let cmp = $state<Record<number, boolean>>({});
  const label: Record<string, string> = { queued: "Waiting", running: "Encoding", done: "Done", failed: "Failed", cancelled: "Cancelled" };

  $effect(() => {
    for (const j of jobs) if (j.preset.video) app.loadThumb(j.source, 1);
  });

  function delta(j: Job): { text: string; worse: boolean } | null {
    const a = j.source_size, b = j.verify?.size_bytes;
    if (!a || !b) return null;
    // A trimmed export is shorter than the file it came from, so a percentage would be misleading.
    if (j.start_secs != null || j.end_secs != null) return { text: `${fmtBytes(b)} for the selected range`, worse: false };
    const pct = ((b - a) / a) * 100;
    return { text: `${fmtBytes(a)} → ${fmtBytes(b)} (${pct > 0 ? "+" : "−"}${Math.abs(pct).toFixed(0)}%)`, worse: pct > 0 };
  }
  async function copy(j: Job) {
    try {
      await navigator.clipboard.writeText(app.report(j));
      app.say("Report copied to the clipboard");
    } catch {
      app.say("Could not copy — open Details and select the text instead", "error");
    }
  }
  const running = $derived(jobs.filter((j) => j.state === "running").length);
  const waiting = $derived(jobs.filter((j) => j.state === "queued").length);
  const okCount = $derived(jobs.filter((j) => j.state === "done").length);
  const badCount = $derived(jobs.filter((j) => j.state === "failed").length);
</script>

<section class="queue" aria-label={kind === "active" ? "Active jobs" : "Completed jobs"}>
  <div class="bar">
    {#if kind === "active"}
      <div class="sum">
        <strong>{running} encoding</strong><span class="muted"> · {waiting} waiting</span>
        {#if app.overall}
          <div class="obar" role="progressbar" aria-label="Overall progress" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(app.overall.percent)}><div class="ofill" style="width:{app.overall.percent}%"></div></div>
          <span class="muted">{app.overall.finished}/{app.overall.total} · {Math.round(app.overall.percent)}%</span>
        {/if}
      </div>
      <span class="grow"></span>
      <button class="btn" onclick={async () => { await app.run(api.queuePause); app.say("Paused — running jobs finish, waiting jobs are held"); }}>Pause</button>
      <button class="btn" onclick={async () => { await app.run(api.queueStart); app.say("Resumed"); }}>Resume</button>
      <button class="btn" onclick={() => app.run(api.queueCancelAll)}>Cancel all</button>
    {:else}
      <div class="sum"><strong>{okCount} done</strong>{#if badCount}<span class="bad"> · {badCount} failed</span>{/if}<span class="muted"> · {jobs.length} total</span></div>
      <span class="grow"></span>
      <button class="btn" disabled={!okCount} title="Save a JSON list of what was made, with sizes, checks and SHA-256" onclick={async () => { const p = await app.run(api.exportManifest); if (p) app.say(`Manifest saved: ${p}`); }}>Export manifest…</button>
      <button class="btn" disabled={!jobs.length} onclick={() => app.run(api.queueClearFinished)}>Clear finished</button>
    {/if}
  </div>

  <div class="list">
    {#if jobs.length === 0}
      {@const justDone = kind === "active" && app.overall && app.overall.finished === app.overall.total}
      <div class="empty">
        {#if justDone}
          <strong>Batch finished</strong>
          <span>{app.overall?.total} job(s) are done. Open Completed to check the results.</span>
          <button class="btn primary" onclick={() => (app.tab = "completed")}>View results</button>
        {:else}
          <strong>{kind === "active" ? "Nothing is running" : "No finished jobs yet"}</strong>
          <span>{kind === "active" ? "Add files on the Current tab, pick presets and press Start batch." : "Finished, failed and cancelled jobs appear here with their results."}</span>
          <button class="btn" onclick={() => (app.tab = "current")}>Go to Current</button>
        {/if}
      </div>
    {/if}

    {#each jobs as j (j.id)}
      {@const d = delta(j)}
      <article class="job" class:sel={app.selectedJob === j.id} data-state={j.state}>
        <button class="main" onclick={() => (app.selectedJob = j.id)} aria-pressed={app.selectedJob === j.id}>
          <span class="thumb">{#if app.thumbs[j.source]}<img src={app.thumbs[j.source]} alt="" />{:else if !j.preset.video}<span aria-hidden="true">♪</span>{/if}</span>
          <span class="txt">
            <span class="t1"><strong>{j.file_name}</strong><span class="arrow">→</span><span class="pre">{j.preset.name}</span></span>
            {#if j.start_secs != null || j.end_secs != null}<span class="mark">Range {fmtDuration(j.start_secs ?? 0)} – {j.end_secs != null ? fmtDuration(j.end_secs) : "end"}</span>{/if}
          </span>
          <span class="st {j.state}">{label[j.state]}</span>
        </button>

        {#if j.state === "running" || j.state === "queued"}
          <div class="meter" role="progressbar" aria-label="Progress of {j.file_name}" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(j.progress)}>
            <div class="fill" class:wait={j.state === "queued"} style="width:{j.progress}%"></div>
          </div>
          <div class="stats">
            {#if j.state === "running" && j.progress >= 99.9}
              <span><b>Finalizing…</b></span>
              <span>writing the file and checking it with ffprobe</span>
            {:else if j.state === "running"}
              <span><b>{Math.round(j.progress)}%</b></span>
              {#if j.fps}<span>{j.fps.toFixed(0)} fps</span>{/if}
              {#if j.speed}<span>{j.speed.toFixed(2)}× speed</span>{/if}
              {#if app.elapsed(j) != null}<span>{fmtDuration(app.elapsed(j))} elapsed</span>{/if}
              {#if j.eta_secs != null}<span>{fmtEta(j.eta_secs)}</span>{/if}
            {:else}
              <span>Waiting for a free slot</span>
            {/if}
            {#if j.note}<span class="note">{j.note}</span>{/if}
          </div>
        {:else if j.state === "done"}
          <div class="result">
            {#if j.verify?.ok}<span class="ok">✓ Verified</span>{:else if j.verify}<span class="warn">⚠ Check output</span>{:else}<span class="muted">Not verified</span>{/if}
            {#if d}<span class:worse={d.worse}>{d.text}</span>{/if}
            {#if j.verify?.duration_secs != null}<span>{fmtDuration(j.verify.duration_secs)}</span>{/if}
            {#if j.verify?.video_codec}<span>{j.verify.video_codec}{j.verify.width ? ` ${j.verify.width}×${j.verify.height}` : ""}</span>{/if}
            {#if j.verify?.audio_codec}<span>{j.verify.audio_codec}</span>{/if}
            {#if app.elapsed(j) != null}<span class="muted">took {fmtDuration(app.elapsed(j))}</span>{/if}
          </div>
          {#if j.verify && !j.verify.ok}<div class="problem">{j.verify.notes.join(" · ")}</div>{/if}
          {#if d?.worse}<div class="problem soft">The output is larger than the source — it was already well compressed. Try a “smaller file” preset or HEVC.</div>{/if}
          {#if j.note}<div class="path">{j.note}</div>{/if}
          {#if j.output}<div class="path" title={j.output}>{j.output}</div>{/if}
          {#if j.verify?.sha256}<div class="path" title="SHA-256 of the exported file">SHA-256 {j.verify.sha256.slice(0, 16)}…</div>{/if}
        {:else if j.error}
          <div class="problem">{j.error}</div>
        {/if}

        <div class="acts">
          {#if j.state === "running" || j.state === "queued"}
            <button class="btn" onclick={() => app.run(() => api.jobCancel(j.id))}>Cancel</button>
          {/if}
          {#if j.state === "failed" || j.state === "cancelled"}
            <button class="btn" onclick={async () => { await app.run(() => api.jobRetry(j.id)); await app.run(api.queueStart); app.tab = "active"; }}>Retry</button>
          {/if}
          {#if j.state === "done"}
            <button class="btn" onclick={() => app.run(() => api.revealOutput(j.id))}>Show in folder</button>
            {#if j.preset.video && j.output}
              <button class="btn" onclick={() => (cmp[j.id] = !cmp[j.id])} aria-expanded={!!cmp[j.id]}>Compare {cmp[j.id] ? "▴" : "▾"}</button>
            {/if}
          {/if}
          {#if j.state !== "queued"}
            <button class="btn" onclick={() => (open[j.id] = !open[j.id])} aria-expanded={!!open[j.id]}>Details {open[j.id] ? "▴" : "▾"}</button>
          {/if}
          {#if j.state !== "queued" && j.state !== "running"}
            <button class="btn" onclick={() => copy(j)}>Copy report</button>
          {/if}
          <span class="grow"></span>
          {#if j.state !== "running"}
            <button class="btn" onclick={() => app.run(() => api.jobRemove(j.id))}>Remove</button>
          {/if}
        </div>
        {#if cmp[j.id] && j.output}
          <Compare job={j} />
        {/if}
        {#if open[j.id]}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
          <pre class="details" tabindex="0">{app.report(j)}</pre>
        {/if}
      </article>
    {/each}
  </div>
</section>

<style>
  .queue { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .bar { flex: none; display: flex; align-items: center; gap: 8px; padding: 6px 12px; background: var(--panel); border-bottom: 1px solid var(--line-soft); }
  .sum { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .obar { width: 160px; height: 6px; background: var(--panel-3); border-radius: 3px; overflow: hidden; margin-left: 6px; }
  .ofill { height: 100%; background: var(--accent); transition: width 0.3s; }
  .grow { flex: 1; }
  .bad { color: var(--danger); }
  .list { flex: 1; min-height: 0; overflow: auto; padding: 8px 10px 14px; display: flex; flex-direction: column; gap: 8px; }
  .empty { display: grid; justify-items: center; gap: 8px; padding: 48px 20px; color: var(--faint); text-align: center; }
  .empty strong { color: var(--muted); font-size: 13px; }
  .job { background: var(--panel); border: 1px solid var(--line-soft); border-radius: 7px; padding: 8px 10px; display: grid; gap: 6px; }
  .job.sel { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent-soft); }
  .job[data-state="failed"] { border-color: #ff5f5766; }
  .main { display: flex; align-items: center; gap: 10px; background: none; border: 0; padding: 0; text-align: left; min-width: 0; }
  .thumb { width: 56px; height: 32px; flex: none; background: #000; border-radius: 3px; overflow: hidden; display: grid; place-items: center; color: var(--muted); }
  .thumb img { width: 100%; height: 100%; object-fit: cover; }
  .txt { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .t1 { display: flex; gap: 8px; align-items: baseline; min-width: 0; }
  .t1 strong { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .arrow { color: var(--faint); }
  .pre { color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .mark { color: var(--marker); font-size: 11px; }
  .st { font-size: 11px; padding: 1px 9px; border-radius: 9px; background: var(--panel-3); color: var(--muted); }
  .st.running { background: var(--accent); color: var(--accent-ink); }
  .st.done { background: #1f6f43; color: #d6ffe6; }
  .st.failed { background: var(--danger); color: #fff; }
  .meter { height: 6px; background: var(--panel-3); border-radius: 3px; overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 0.25s; }
  .fill.wait { background: var(--faint); }
  .stats, .result { display: flex; flex-wrap: wrap; gap: 4px 14px; color: var(--muted); font-variant-numeric: tabular-nums; }
  .stats b { color: var(--text); }
  .note { color: var(--faint); }
  .ok { color: #4cd07d; font-weight: 600; }
  .warn { color: var(--warn); font-weight: 600; }
  .worse { color: var(--warn); }
  .problem { color: var(--danger); background: #ff5f5714; border-radius: 5px; padding: 4px 8px; }
  .problem.soft { color: var(--warn); background: #f5a62314; }
  .path { color: var(--faint); font-size: 11px; word-break: break-all; }
  .acts { display: flex; gap: 6px; align-items: center; }
  .acts .btn { padding: 2px 10px; }
  .details { margin: 0; max-height: 240px; overflow: auto; background: var(--bg); border: 1px solid var(--line-soft); border-radius: 5px; padding: 8px 10px; font: 11px/1.5 "Cascadia Mono", Consolas, monospace; white-space: pre-wrap; word-break: break-all; user-select: text; }
</style>
