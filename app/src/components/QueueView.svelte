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
  .queue { overflow: auto; padding: 10px 12px; display: grid; gap: 10px; align-content: start; grid-row: 1 / span 2; }
  .bar { display: flex; gap: 8px; }
  .bar button, .acts button { background: var(--panel-2); border: 1px solid var(--line); border-radius: 6px; padding: 4px 12px; }
  .empty { color: var(--muted); padding: 24px; text-align: center; }
  .job { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 8px 12px; display: grid; gap: 4px; }
  .job.sel { border-color: var(--accent); }
  .head { background: none; border: 0; text-align: left; padding: 0; display: flex; gap: 8px; align-items: baseline; }
  .p { color: var(--muted); flex: 1; }
  .st { font-size: 12px; padding: 1px 8px; border-radius: 10px; background: var(--panel-2); }
  .st.running { background: var(--accent); color: var(--accent-ink); }
  .st.failed { background: var(--danger); color: #fff; }
  .meter { height: 8px; background: var(--panel-2); border-radius: 4px; overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 0.25s; }
  small { color: var(--muted); word-break: break-all; }
  .err { color: var(--danger); }
  .acts { display: flex; gap: 8px; margin-top: 2px; }
</style>
