<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { api } from "$lib/api";
  import { fmtEta } from "$lib/format";
  import type { Job } from "$lib/types";

  let { jobs, kind }: { jobs: Job[]; kind: "active" | "completed" } = $props();
  const label: Record<string, string> = { queued: "Waiting", running: "Encoding", done: "Done", failed: "Failed", cancelled: "Cancelled" };
</script>

<section class="queue" aria-label={kind === "active" ? "Active jobs" : "Completed jobs"}>
  <div class="bar">
    {#if kind === "active"}
      <button onclick={() => app.run(api.queuePause)}>Pause queue</button>
      <button onclick={() => app.run(api.queueStart)}>Resume</button>
      <button onclick={() => app.run(api.queueCancelAll)}>Cancel all</button>
    {:else}
      <button onclick={() => app.run(api.queueClearFinished)}>Clear finished</button>
    {/if}
  </div>
  {#if jobs.length === 0}
    <p class="empty">{kind === "active" ? "Nothing is running." : "Nothing finished yet."}</p>
  {/if}
  {#each jobs as j (j.id)}
    <div class="job" class:sel={app.selectedJob === j.id} data-state={j.state}>
      <button class="head" onclick={() => (app.selectedJob = j.id)}>
        <strong>{j.file_name}</strong> <span class="p">→ {j.preset.name}</span>
        <span class="st {j.state}">{label[j.state]}</span>
      </button>
      {#if j.state === "running" || j.state === "queued"}
        <div class="meter" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(j.progress)} aria-label="Progress of {j.file_name}">
          <div class="fill" style="width:{j.progress}%"></div>
        </div>
        <small>{Math.round(j.progress)}%{j.fps ? ` · ${j.fps.toFixed(0)} fps` : ""}{j.speed ? ` · ${j.speed.toFixed(2)}×` : ""} {fmtEta(j.eta_secs)}</small>
      {/if}
      {#if j.note}<small class="note">{j.note}</small>{/if}
      {#if j.error}<small class="err">{j.error}</small>{/if}
      {#if j.output && j.state === "done"}<small class="out">{j.output}</small>{/if}
      <div class="acts">
        {#if j.state === "running" || j.state === "queued"}
          <button onclick={() => app.run(() => api.jobCancel(j.id))}>Cancel</button>
        {/if}
        {#if j.state === "failed" || j.state === "cancelled"}
          <button onclick={async () => { await app.run(() => api.jobRetry(j.id)); await app.run(api.queueStart); }}>Retry</button>
        {/if}
        {#if j.state === "done"}
          <button onclick={() => app.run(() => api.revealOutput(j.id))}>Show in folder</button>
        {/if}
        {#if j.state !== "running"}
          <button onclick={() => app.run(() => api.jobRemove(j.id))}>Remove</button>
        {/if}
      </div>
    </div>
  {/each}
</section>

<style>
  .queue { flex: 1; min-height: 0; overflow: auto; display: flex; flex-direction: column; }
  .bar { position: sticky; top: 0; z-index: 1; display: flex; gap: 6px; padding: 5px 10px; background: var(--panel); border-bottom: 1px solid var(--line-soft); }
  .bar button, .acts button { background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; padding: 2px 10px; }
  .bar button:hover, .acts button:hover { background: var(--panel-3); }
  .empty { color: var(--faint); padding: 28px; text-align: center; }
  .job { margin: 8px 10px 0; background: var(--panel); border: 1px solid var(--line-soft); border-radius: 7px; padding: 7px 10px; display: grid; gap: 4px; }
  .job.sel { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent-soft); }
  .head { background: none; border: 0; text-align: left; padding: 0; display: flex; gap: 8px; align-items: baseline; min-width: 0; }
  .head strong { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .p { color: var(--muted); flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .st { font-size: 11px; padding: 0 8px; border-radius: 9px; background: var(--panel-3); color: var(--muted); }
  .st.running { background: var(--accent); color: var(--accent-ink); }
  .st.done { background: #1f6f43; color: #d6ffe6; }
  .st.failed { background: var(--danger); color: #fff; }
  .meter { height: 5px; background: var(--panel-3); border-radius: 3px; overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 0.25s; }
  small { color: var(--muted); font-size: 11px; word-break: break-all; }
  .err { color: var(--danger); }
  .acts { display: flex; gap: 6px; margin-top: 2px; }
</style>
