// E2E for the P0 feature set, driving the real exe through WebView2 DevTools.
// Usage: node scripts/e2e-p0.mjs <squeeze.exe> [screenshot-dir]
import { spawn, execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const exe = process.argv[2];
const shots = process.argv[3] ?? join(tmpdir(), "squeeze-p0-shots");
if (!exe || !existsSync(exe)) throw new Error("usage: node e2e-p0.mjs <app.exe>");
mkdirSync(shots, { recursive: true });
const PORT = 9338;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const work = mkdtempSync(join(tmpdir(), "squeeze-p0-"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push(!!ok);
  console.log(`${ok ? "PASS" : "FAIL"}  ${name}${detail ? "  — " + String(detail).slice(0, 260) : ""}`);
};
const ff = (args) => execFileSync("ffmpeg", ["-v", "error", "-y", ...args], { stdio: "pipe" });
const probeJson = (f) => JSON.parse(execFileSync("ffprobe", ["-v", "error", "-print_format", "json", "-show_format", "-show_streams", f]).toString());

// ---- fixtures
const V = "testsrc2=size=640x360:rate=30:duration=3";
const twoTracks = join(work, "two_tracks.mkv");
ff(["-f", "lavfi", "-i", V, "-f", "lavfi", "-i", "sine=frequency=300:duration=3:sample_rate=8000", "-f", "lavfi", "-i", "sine=frequency=900:duration=3:sample_rate=44100", "-map", "0:v", "-map", "1:a", "-map", "2:a", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-ac:a:0", "1", "-ac:a:1", "2", "-metadata:s:a:1", "language=tha", "-metadata:s:a:1", "title=Thai dub", "-shortest", twoTracks]);
const hdr = join(work, "hdr10.mkv");
ff(["-f", "lavfi", "-i", V, "-c:v", "libx265", "-pix_fmt", "yuv420p10le", "-x265-params", "colorprim=bt2020:transfer=smpte2084:colormatrix=bt2020nc:log-level=error", hdr]);
const vfr = join(work, "vfr.mp4");
ff(["-f", "lavfi", "-i", "testsrc2=size=640x360:rate=60:duration=3", "-vf", "select='not(mod(n,3))+not(mod(n,7))'", "-fps_mode", "vfr", "-c:v", "libx264", "-pix_fmt", "yuv420p", vfr]);
const srt = join(work, "s.srt"); writeFileSync(srt, "1\n00:00:00,000 --> 00:00:01,000\nสวัสดี\n");
const subs = join(work, "subs.mp4");
ff(["-f", "lavfi", "-i", V, "-i", srt, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:s", "mov_text", "-metadata:s:s:0", "language=tha", "-shortest", subs]);

// ---- CDP plumbing
let app = null, ws = null, id = 0;
const pend = new Map();
async function launch() {
  app = spawn(exe, [], { env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` }, stdio: "ignore" });
  ws = null;
  for (let i = 0; i < 80 && !ws; i++) {
    try {
      const p = (await (await fetch(`http://127.0.0.1:${PORT}/json`)).json()).find((t) => t.type === "page");
      if (p) { ws = new WebSocket(p.webSocketDebuggerUrl); await new Promise((r) => (ws.onopen = r)); ws.onmessage = (m) => { const d = JSON.parse(m.data); pend.get(d.id)?.(d); }; }
    } catch {}
    if (!ws) await sleep(500);
  }
  if (!ws) throw new Error("no devtools");
  await send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
  await sleep(2500);
}
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pend.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async (e) => {
  const r = await send("Runtime.evaluate", { expression: e, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails.exception?.description ?? r.result.exceptionDetails).slice(0, 300));
  return r.result?.result?.value;
};
const inv = (c, a = {}) => ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(c)}, ${JSON.stringify(a)}).then(v=>({ok:true,v}),e=>({ok:false,e:String(e)}))`);
const text = () => ev("document.body?.innerText ?? ''");
const shot = async (n) => writeFileSync(join(shots, n), Buffer.from((await send("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
const click = (sel, txt) => ev(`(() => { const el=[...document.querySelectorAll(${JSON.stringify(sel)})].find(e=>e.innerText.trim().includes(${JSON.stringify(txt)})); if(!el) return false; el.click(); return true })()`);
async function waitFor(fn, ms = 60000, step = 200) { const t0 = Date.now(); while (Date.now() - t0 < ms) { const v = await fn(); if (v) return v; await sleep(step); } return false; }
const drop = (paths) => ev(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'tauri://drag-drop',payload:{paths:${JSON.stringify(paths)},position:{x:1,y:1}}})`);
const select = (sel, valueOrText) => ev(`(() => { const el=document.querySelector(${JSON.stringify(sel)}); if(!el) return false; const o=[...el.options].find(o=>o.value===${JSON.stringify(String(valueOrText))}||o.textContent.includes(${JSON.stringify(String(valueOrText))})); if(!o) return false; el.value=o.value; el.dispatchEvent(new Event('change',{bubbles:true})); el.dispatchEvent(new Event('input',{bubbles:true})); return true })()`);
const openGroups = async () => { await ev(`[...document.querySelectorAll('.group[aria-expanded="false"]')].forEach(g=>g.click())`); await sleep(150); };
const clearBatch = () => ev(`(() => { [...document.querySelectorAll('.source .src-head .icon-btn')].forEach(b=>b.click()); })()`);

try {
  await launch();
  await waitFor(async () => (await text()).includes("Squeeze"), 20000);
  await inv("queue_cancel_all");

  // ============ Inspector detail ============
  await click("[role=tab]", "Current");
  await drop([twoTracks]);
  await waitFor(async () => (await text()).includes("two_tracks.mkv"), 20000);
  await sleep(800);
  let t = await text();
  check("inspector lists both audio streams", t.includes("Audio (2 streams)") && /#1 · aac · mono/.test(t) && /#2 · aac · stereo/.test(t) && t.includes("Thai dub"), t.match(/Audio \(2[\s\S]{0,200}/)?.[0]?.replace(/\s+/g, " "));
  check("video details shown (profile, bit depth, constant frame rate)", /Profile \/ level/.test(t) && /8-bit/.test(t) && /constant/.test(t) && /Unknown/.test(t), "");
  await clearBatch();
  await sleep(300);

  await drop([hdr]);
  await waitFor(async () => (await text()).includes("hdr10.mkv"), 20000); await sleep(800);
  t = await text();
  check("HDR source is detected and explained", /HDR source \(smpte2084\)/.test(t) && /10-bit/.test(t) && /Main 10/.test(t) && /bt2020/.test(t), t.match(/HDR source[\s\S]{0,120}/)?.[0]);
  await shot("p0-1-hdr.png");
  await clearBatch(); await sleep(300);

  await drop([vfr]);
  await waitFor(async () => (await text()).includes("vfr.mp4"), 20000); await sleep(800);
  t = await text();
  check("variable frame rate is flagged", /variable/.test(t) && /Variable frame rate/.test(t), "");
  await clearBatch(); await sleep(300);

  await drop([subs]);
  await waitFor(async () => (await text()).includes("subs.mp4"), 20000); await sleep(800);
  t = await text();
  check("subtitle track is listed", /Subtitles/.test(t) && /mov_text/.test(t), "");
  await clearBatch(); await sleep(300);

  // ============ Audio: stream choice, copy, conflicts ============
  await drop([twoTracks]);
  await waitFor(async () => (await text()).includes("two_tracks.mkv"), 20000); await sleep(800);
  await ev(`document.querySelectorAll('input[name=astream]')[1].click()`);
  await select('select[aria-label="What to do with the audio"]', "copy");
  await sleep(300);
  // add a WebM output: AAC cannot be copied into WebM -> Start must be blocked with the reason
  await click(".leaf", "VP9 WebM");
  await sleep(400);
  const hint = await ev(`document.querySelector('.bottom .hint')?.innerText ?? ''`);
  const blocked = await ev(`document.querySelector('.bottom .btn.primary').disabled`);
  check("copying AAC into WebM is blocked before starting, with the reason", blocked && /cannot be copied into \.webm/.test(hint), hint);
  // remove the VP9 output row (last row), leaving the mp4 one
  await ev(`(() => { const rows=[...document.querySelectorAll('.source.sel tbody tr')]; rows[rows.length-1].querySelector('.icon-btn').click(); })()`);
  await sleep(400);
  check("blocking clears once the conflicting output is removed", !(await ev(`document.querySelector('.bottom .btn.primary').disabled`)), "");
  // "remove audio" for an audio-only preset is refused too
  await openGroups();
  await click(".leaf", "AAC 256");
  await select('select[aria-label="What to do with the audio"]', "none");
  await sleep(300);
  const hint2 = await ev(`document.querySelector('.bottom .hint')?.innerText ?? ''`);
  check("removing audio with an audio-only preset is blocked", /audio-only/.test(hint2), hint2);
  await select('select[aria-label="What to do with the audio"]', "copy"); await sleep(200);
  await ev(`(() => { const rows=[...document.querySelectorAll('.source.sel tbody tr')]; rows[rows.length-1].querySelector('.icon-btn').click(); })()`); // drop the m4a row
  await sleep(300);
  const before = (await inv("queue_list")).v.map((j) => j.id);
  await click(".primary", "Start batch");
  const copyJob = await waitFor(async () => { const js = (await inv("queue_list")).v.filter((j) => !before.includes(j.id)); return js.length === 1 && ["done", "failed"].includes(js[0].state) ? js[0] : false; }, 60000);
  check("copy of audio stream #2 finished and verified", copyJob && copyJob.state === "done" && copyJob.verify?.ok, copyJob?.error ?? JSON.stringify(copyJob?.verify?.notes));
  if (copyJob?.output) {
    const pj = probeJson(copyJob.output);
    const a = pj.streams.filter((s) => s.codec_type === "audio");
    check("output carries only the chosen track, copied untouched (stereo, 44.1 kHz AAC)", a.length === 1 && a[0].codec_name === "aac" && a[0].channels === 2 && a[0].sample_rate === "44100", JSON.stringify(a.map((s) => [s.codec_name, s.channels, s.sample_rate])));
    check("job records the audio choice and its command", copyJob.audio.mode === "copy" && copyJob.audio.stream === 1 && copyJob.cmdline.includes("0:a:1") && copyJob.cmdline.includes("-c:a copy"), copyJob.cmdline?.slice(0, 80));
    // SHA-256 equals Windows' own
    const ps = execFileSync("powershell", ["-NoProfile", "-Command", `(Get-FileHash -Algorithm SHA256 -LiteralPath '${copyJob.output}').Hash.ToLower()`]).toString().trim();
    check("SHA-256 in the report equals Windows Get-FileHash", copyJob.verify.sha256 === ps, `${copyJob.verify.sha256?.slice(0, 12)} / ${ps.slice(0, 12)}`);
  }

  // ============ Completed: compare + report ============
  await click("[role=tab]", "Completed"); await sleep(600);
  await click(".acts .btn", "Compare");
  const cmpOk = await waitFor(async () => (await ev(`document.querySelectorAll('.cmp .stage img').length`)) === 2, 15000);
  check("A/B compare shows a frame from the source and one from the export", cmpOk, "");
  await sleep(500);
  await shot("p0-2-compare.png");
  const wipeBefore = await ev(`document.querySelector('.cmp .bar')?.style.left`);
  await ev(`(() => { const r=document.querySelector('.cmp input[aria-label="Divider between source and output"]'); r.value=20; r.dispatchEvent(new Event('input',{bubbles:true})); })()`);
  await sleep(200);
  check("the wipe divider moves", (await ev(`document.querySelector('.cmp .bar')?.style.left`)) !== wipeBefore, `${wipeBefore} -> 20%`);
  check("report shows SHA-256", (await text()).includes("SHA-256"), "");
  check("manifest button is enabled when something finished", !(await ev(`[...document.querySelectorAll('.bar .btn')].find(b=>b.innerText.includes('Export manifest')).disabled`)), "");
  await click("[role=tab]", "Current");

  // ============ Presets v2: favourites, duplicate, advanced dialog ============
  await clearBatch(); await sleep(300);
  await click(".leaf", "H.264 MP4 — 720p"); // no file yet -> message only
  await ev(`(() => { const row=[...document.querySelectorAll('.row')].find(r=>r.innerText.includes('H.264 MP4 — 720p')); row.querySelector('[title="Favourite"]').click(); })()`);
  await sleep(600);
  check("starring a preset creates a Favourites section", /favourites/i.test(await text()) && (await inv("get_settings")).v.favorites.includes("h264-720"), JSON.stringify((await inv("get_settings")).v.favorites));
  await ev(`(() => { const row=[...document.querySelectorAll('.row')].find(r=>r.innerText.includes('H.264 MP4 — 720p') && r.querySelector('[title="Duplicate to Custom"]')); row.querySelector('[title="Duplicate to Custom"]').click(); })()`);
  await sleep(800);
  const copies = (await inv("list_presets")).v.filter((p) => p.group === "Custom");
  check("duplicating a built-in preset creates an editable Custom copy", copies.length === 1 && /copy/.test(copies[0].name) && copies[0].schema === 2, JSON.stringify(copies.map((p) => [p.id, p.name, p.schema])));
  await inv("delete_custom_preset", { id: copies[0].id });
  // legacy (schema-less) preset JSON still loads through the backend
  const legacy = { id: "legacy-1", name: "Legacy", group: "x", container: "mp4", video: { codec: "h264", rate: { mode: "crf", value: 24 }, max_height: 480, fps: null }, audio: { codec: "aac", bitrate_kbps: 96, channels: null } };
  const sv = await inv("save_custom_preset", { preset: legacy });
  check("a version-1 preset (no schema, no advanced fields) is accepted and upgraded", sv.ok && sv.v.find((p) => p.id === "legacy-1")?.schema === 2, sv.e);
  await inv("delete_custom_preset", { id: "legacy-1" });

  // build an advanced preset through the dialog UI
  await click("[aria-label='New custom preset']", "") || await ev(`document.querySelector('[aria-label="New custom preset"]').click()`);
  await sleep(500);
  await ev(`(() => { const el=document.querySelector('.dlg input[maxlength="80"]'); el.value='UI advanced'; el.dispatchEvent(new Event('input',{bubbles:true})); })()`);
  await ev(`document.querySelector('.dlg details.adv summary').click()`);
  await sleep(200);
  const dlgSel = (label, val) => ev(`(() => { const l=[...document.querySelectorAll('.dlg label')].find(x=>x.innerText.trim().startsWith(${JSON.stringify(label)})); const el=l.querySelector('select'); const o=[...el.options].find(o=>o.value===${JSON.stringify(val)}); el.value=o.value; el.dispatchEvent(new Event('change',{bubbles:true})); el.dispatchEvent(new Event('input',{bubbles:true})); return true })()`);
  await dlgSel("Encoder effort", "fast");
  await dlgSel("H.264 profile", "high");
  await dlgSel("H.264 level", "4.1");
  await ev(`(() => { const l=[...document.querySelectorAll('.dlg label')].find(x=>x.innerText.trim().startsWith('Keyframe every')); const el=l.querySelector('input'); el.value='1'; el.dispatchEvent(new Event('input',{bubbles:true})); })()`);
  await shot("p0-3-dialog.png");
  await click(".dlg .primary", "Save");
  await sleep(800);
  const adv = (await inv("list_presets")).v.find((p) => p.name === "UI advanced");
  check("advanced settings are saved from the dialog", adv && adv.video.profile === "high" && adv.video.level === "4.1" && adv.video.speed === "fast" && adv.video.keyint_secs === 1, JSON.stringify(adv?.video));
  if (adv) {
    await drop([subs]);
    await waitFor(async () => (await text()).includes("subs.mp4"), 20000); await sleep(600);
    await ev(`(() => { const rows=[...document.querySelectorAll('.source.sel tbody tr')]; rows.forEach(r=>r.querySelector('.icon-btn').click()); })()`);
    await sleep(300);
    await click(".leaf", "UI advanced");
    await sleep(300);
    const known = (await inv("queue_list")).v.map((j) => j.id);
    await click(".primary", "Start batch");
    const advJob = await waitFor(async () => { const js = (await inv("queue_list")).v.filter((j) => !known.includes(j.id)); return js.length === 1 && ["done", "failed"].includes(js[0].state) ? js[0] : false; }, 90000);
    check("the advanced preset runs, in software, and verifies", advJob && advJob.state === "done" && advJob.verify?.ok && /software/.test(advJob.note ?? ""), advJob?.error ?? advJob?.note);
    if (advJob?.output) {
      const pj = probeJson(advJob.output);
      const v = pj.streams.find((s) => s.codec_type === "video");
      check("independent ffprobe: H.264 High profile, level 4.1", v.profile === "High" && v.level === 41, `${v.profile} ${v.level}`);
    }
    await inv("delete_custom_preset", { id: adv.id });
  }

  // ============ Estimates ============
  await clearBatch(); await sleep(300);
  await drop([twoTracks]);
  await waitFor(async () => (await text()).includes("two_tracks.mkv"), 20000); await sleep(600);
  await openGroups();
  await click(".leaf", "AAC 256");
  await sleep(400);
  const estCells = await ev(`[...document.querySelectorAll('.source.sel tbody td.c5')].map(c=>c.innerText.trim())`);
  check("estimated size: '—' for quality-based video, '~…' for bitrate audio", estCells.length === 2 && estCells[0] === "—" && /^~/.test(estCells[1]), JSON.stringify(estCells));

  // ============ metadata option ============
  await click(".side [role=tab]", "Locations"); await sleep(300);
  await ev(`document.querySelector('[aria-label="Remove metadata from outputs"]').click()`);
  await sleep(600);
  check("remove-metadata option is saved", (await inv("get_settings")).v.strip_metadata === true, "");
  await ev(`document.querySelector('[aria-label="Remove metadata from outputs"]').click()`);
  await sleep(400);

  await shot("p0-4-final.png");
  // cleanup
  await inv("queue_cancel_all");
  for (const j of (await inv("queue_list")).v) { if (j.output) { try { rmSync(j.output, { force: true }); } catch {} } await inv("job_remove", { id: j.id }); }
} catch (e) {
  check("script completed without exception", false, e);
} finally {
  try { ws?.close(); } catch {}
  try { execFileSync("taskkill", ["/F", "/PID", String(app.pid)], { stdio: "ignore" }); } catch {}
  try { rmSync(work, { recursive: true, force: true }); } catch {}
}
const bad = results.filter((r) => !r).length;
console.log(`\n${results.length - bad}/${results.length} checks passed`);
process.exit(bad ? 1 : 0);
