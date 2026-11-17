// End-to-end test: launches the real Squeeze exe and drives it through WebView2's DevTools Protocol.
// Usage: node scripts/e2e.mjs <path-to-app.exe> [screenshot-dir]
import { spawn, execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync, existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const exe = process.argv[2];
const shots = process.argv[3] ?? join(tmpdir(), "squeeze-e2e-shots");
if (!exe || !existsSync(exe)) throw new Error("usage: node e2e.mjs <app.exe>");
mkdirSync(shots, { recursive: true });
const PORT = 9333;
const work = mkdtempSync(join(tmpdir(), "squeeze-e2e-"));
const results = [];
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
function check(name, ok, detail = "") {
  results.push({ name, ok: !!ok, detail });
  console.log(`${ok ? "PASS" : "FAIL"}  ${name}${detail ? "  — " + detail : ""}`);
}
const ff = (args) => execFileSync("ffmpeg", ["-v", "error", "-y", ...args], { stdio: "pipe" });
const probeJson = (f) => JSON.parse(execFileSync("ffprobe", ["-v", "error", "-print_format", "json", "-show_format", "-show_streams", f]).toString());

function launch() {
  const env = { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` };
  return spawn(exe, [], { env, stdio: "ignore" });
}

async function connect() {
  for (let i = 0; i < 60; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
      const page = list.find((t) => t.type === "page");
      if (page) {
        const ws = new WebSocket(page.webSocketDebuggerUrl);
        await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
        let id = 0;
        const pending = new Map();
        ws.onmessage = (m) => {
          const d = JSON.parse(m.data);
          if (d.id && pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); }
        };
        const send = (method, params = {}) => new Promise((res) => { const i2 = ++id; pending.set(i2, res); ws.send(JSON.stringify({ id: i2, method, params })); });
        return {
          ws,
          async eval(expr) {
            const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
            if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails.exception?.description ?? r.result.exceptionDetails));
            return r.result?.result?.value;
          },
          async shot(name) {
            const r = await send("Page.captureScreenshot", { format: "png" });
            writeFileSync(join(shots, name), Buffer.from(r.result.data, "base64"));
          },
        };
      }
    } catch {}
    await sleep(500);
  }
  throw new Error("could not connect to WebView2 devtools");
}

const inv = (page, cmd, args = {}) =>
  page.eval(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)}).then(v => ({ok:true, v}), e => ({ok:false, e:String(e)}))`);
const text = (page) => page.eval("document.body.innerText");
async function waitFor(fn, ms = 60000, step = 250) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) { const v = await fn(); if (v) return v; await sleep(step); }
  return false;
}
const clickText = (page, sel, txt) =>
  page.eval(`(() => { const el = [...document.querySelectorAll(${JSON.stringify(sel)})].find(e => e.innerText.trim().includes(${JSON.stringify(txt)})); if (!el) return false; el.click(); return true; })()`);

let app = launch();
let page;
try {
  page = await connect();
  await waitFor(async () => (await page.eval("document.body.innerText")).includes("Squeeze"), 20000);

  // ---- app info / tools
  const info = await inv(page, "app_info");
  check("app_info reports ffmpeg", info.ok && info.v.ffmpeg_path, info.ok ? `${info.v.ffmpeg_version}` : info.e);
  await waitFor(async () => (await inv(page, "app_info")).v.hw_ready, 60000);
  const info2 = await inv(page, "app_info");
  check("hardware detection finished", info2.v.hw_ready, `hw=${JSON.stringify(info2.v.hw)}`);

  // ---- fixtures
  const clip = join(work, "ทดสอบ clip.mp4");
  ff(["-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=30:duration=4", "-f", "lavfi", "-i", "sine=frequency=440:duration=4", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest", clip]);
  const wav = join(work, "tone.wav");
  ff(["-f", "lavfi", "-i", "sine=frequency=330:duration=3", wav]);
  const bad = join(work, "broken.mp4");
  writeFileSync(bad, "not a video at all");

  // ---- drag & drop three files (emits the same event the OS drop produces)
  await page.eval(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {event:'tauri://drag-drop', payload:{paths:${JSON.stringify([clip, wav, bad])}, position:{x:10,y:10}}})`);
  const loaded = await waitFor(async () => (await text(page)).includes("Cannot read this file"), 20000);
  check("dropped files are probed; corrupt one is flagged", loaded);
  const t1 = await text(page);
  check("video and audio sources listed", t1.includes("ทดสอบ clip.mp4") && t1.includes("tone.wav"));
  check("probe shows codec/resolution", t1.includes("h264 1280×720"));
  await sleep(1200);
  await page.shot("01-loaded.png");
  const hasImg = await page.eval("!!document.querySelector('.frame img')");
  check("preview frame is shown for video", hasImg);

  // ---- add a second output via preset tree, then start
  await clickText(page, ".leaf", "AV1 WebM");
  await sleep(300);
  const rows = await page.eval("document.querySelectorAll('.source.sel tbody tr').length");
  check("clicking a preset adds an output row", rows === 2, `rows=${rows}`);
  const startBtn = await page.eval("!document.querySelector('.primary').disabled");
  check("Start batch enabled", startBtn);
  await clickText(page, ".primary", "Start batch");
  const done = await waitFor(async () => {
    const jobs = (await inv(page, "queue_list")).v;
    return jobs.length >= 3 && jobs.every((j) => j.state === "done" || j.state === "failed") ? jobs : false;
  }, 120000);
  await page.shot("02-queue.png");
  check("3 jobs finished", done && done.length === 3 && done.every((j) => j.state === "done"), done ? done.map((j) => `${j.preset.id}:${j.state}`).join(",") : "timeout");
  if (done) {
    const clipJob = done.find((j) => j.preset.id === "av1-1080-webm");
    const p = probeJson(clipJob.output);
    check("AV1 WebM output is valid", p.streams.some((s) => s.codec_name === "av1") && p.format.format_name.includes("webm"), clipJob.output);
    const h = done.find((j) => j.preset.id === "h264-1080");
    const p2 = probeJson(h.output);
    check("H.264 output keeps Thai file name + duration", h.output.includes("ทดสอบ clip-h264-1080") && Math.abs(Number(p2.format.duration) - 4) < 0.4);
    const a = done.find((j) => j.preset.id === "aac-256");
    check("audio-only source got audio preset (.m4a)", a && a.output.endsWith(".m4a") && existsSync(a.output));
    check("no temp part files left", !done.some((j) => existsSync(j.output.replace(/(\.[^.]+)$/, ".squeeze-part$1"))));
  }
  const afterText = await text(page);
  await clickText(page, "[role=tab]", "Current");
  await sleep(300);
  check("corrupt file stayed in the batch with its error", (await text(page)).includes("Cannot read this file"));

  // ---- finished tab + reveal
  await clickText(page, "[role=tab]", "Completed");
  await sleep(300);
  await page.shot("03-completed.png");
  check("Completed tab lists the jobs", (await page.eval("document.querySelectorAll('.job').length")) === 3);

  // ---- cancel a long job
  const long = join(work, "long.mp4");
  ff(["-f", "lavfi", "-i", "testsrc2=size=1920x1080:rate=30:duration=90", "-c:v", "libx264", "-preset", "ultrafast", "-pix_fmt", "yuv420p", long]);
  const add = await inv(page, "add_job", { source: long, presetId: "av1-1080-webm", destDir: null });
  check("add_job accepts a valid file", add.ok, add.e);
  await inv(page, "queue_start");
  await waitFor(async () => (await inv(page, "queue_list")).v.some((j) => j.id === add.v && j.state === "running" && j.progress > 0), 60000);
  await sleep(500);
  const t0 = Date.now();
  await inv(page, "job_cancel", { id: add.v });
  const cancelled = await waitFor(async () => (await inv(page, "queue_list")).v.find((j) => j.id === add.v)?.state === "cancelled", 10000);
  check("cancel stops the running job quickly", cancelled, `${Date.now() - t0} ms`);
  const procs = execFileSync("powershell", ["-NoProfile", "-Command", "(Get-Process ffmpeg -ErrorAction SilentlyContinue | Measure-Object).Count"]).toString().trim();
  check("no ffmpeg process left after cancel", procs === "0", `count=${procs}`);

  // ---- persistence + crash cleanup: kill the app mid-encode
  const add2 = await inv(page, "add_job", { source: long, presetId: "av1-1080-webm", destDir: null });
  await inv(page, "queue_start");
  await waitFor(async () => (await inv(page, "queue_list")).v.some((j) => j.id === add2.v && j.state === "running" && j.progress > 0), 60000);
  execFileSync("taskkill", ["/F", "/PID", String(app.pid)], { stdio: "ignore" });
  await sleep(2500);
  const orphan = execFileSync("powershell", ["-NoProfile", "-Command", "(Get-Process ffmpeg -ErrorAction SilentlyContinue | Measure-Object).Count"]).toString().trim();
  check("killing the app also kills ffmpeg (job object)", orphan === "0", `ffmpeg processes=${orphan}`);
  app = launch();
  page = await connect();
  await waitFor(async () => (await text(page)).includes("Squeeze"), 20000);
  const restored = (await inv(page, "queue_list")).v;
  const r2 = restored.find((j) => j.id === add2.v);
  check("queue restored after a crash; running job re-queued", r2 && r2.state === "queued", r2 ? r2.state : "missing");
  const leftovers = restored.some((j) => j.temp);
  check("stale temp path cleared", !leftovers);
  await inv(page, "job_remove", { id: add2.v });

  // ---- custom preset validation
  const goodP = { id: "t-custom", name: "T custom", group: "x", container: "mkv", video: { codec: "h264", rate: { mode: "crf", value: 23 }, max_height: 480, fps: 24 }, audio: { codec: "flac", bitrate_kbps: null, channels: 2 } };
  const s1 = await inv(page, "save_custom_preset", { preset: goodP });
  check("valid custom preset saved", s1.ok && s1.v.some((p) => p.id === "t-custom" && p.group === "Custom"), s1.e);
  const badP = structuredClone(goodP); badP.container = "webm"; badP.id = "t-bad";
  check("incompatible preset rejected", !(await inv(page, "save_custom_preset", { preset: badP })).ok);
  const evil = structuredClone(goodP); evil.id = "../../evil";
  check("path-like preset id rejected", !(await inv(page, "save_custom_preset", { preset: evil })).ok);
  const shadow = structuredClone(goodP); shadow.id = "h264-1080";
  check("custom preset cannot shadow built-in", !(await inv(page, "save_custom_preset", { preset: shadow })).ok);
  const tmpJob = await inv(page, "add_job", { source: wav, presetId: "t-custom", destDir: null });
  await inv(page, "delete_custom_preset", { id: "t-custom" });

  // ---- hostile inputs to commands
  const h1 = await inv(page, "add_job", { source: "http://example.com/a.mp4", presetId: "h264-1080", destDir: null });
  check("URL as source rejected", !h1.ok, h1.e);
  const h2 = await inv(page, "add_job", { source: wav, presetId: "no-such", destDir: null });
  check("unknown preset id rejected", !h2.ok);
  const h3 = await inv(page, "add_job", { source: wav, presetId: "flac", destDir: "relative\\dir" });
  check("relative destination rejected", !h3.ok, h3.e);
  const h4 = await inv(page, "add_job", { source: work, presetId: "flac", destDir: null });
  check("directory as source rejected", !h4.ok);
  const h5 = await inv(page, "probe_files", { paths: Array.from({ length: 400 }, (_, i) => `C:\\x${i}.mp4`) });
  check("probe batch capped at 300", !h5.ok, h5.e);
  const h6 = await inv(page, "reveal_output", { id: 999999 });
  check("reveal_output rejects unknown job", !h6.ok);
  const h7 = await inv(page, "preview", { path: "file:///C:/Windows/win.ini", secs: 0 });
  check("preview rejects non-file path", !h7.ok, h7.e);
  const h8 = await inv(page, "set_settings", { settings: { max_parallel: 99, use_hw: true, default_dest: "rel", template: "", overwrite: "rename" } });
  check("settings with relative default_dest rejected", !h8.ok, h8.e);
  const h9 = await inv(page, "set_settings", { settings: { max_parallel: 99, use_hw: true, default_dest: null, template: "{name}", overwrite: "rename" } });
  check("max_parallel clamped to 4", h9.ok && h9.v.max_parallel === 4);
  await inv(page, "set_settings", { settings: { max_parallel: 1, use_hw: true, default_dest: null, template: "{name}-{preset}", overwrite: "rename" } });

  // ---- web-layer isolation
  const net = await page.eval(`fetch('https://example.com').then(() => 'allowed', () => 'blocked')`);
  check("CSP blocks outside network requests", net === "blocked", net);
  const csp = await page.eval(`document.querySelector('meta[http-equiv="Content-Security-Policy"]')?.content ?? 'header-based'`);
  console.log("CSP source:", csp.slice(0, 80));
  const fsPerm = await inv(page, "plugin:fs|read_file", { path: "C:\\Windows\\win.ini" });
  check("no filesystem plugin reachable from the page", !fsPerm.ok, fsPerm.e);
  const shellPerm = await inv(page, "plugin:shell|execute", { program: "cmd" });
  check("no shell plugin reachable from the page", !shellPerm.ok, shellPerm.e);

  // ---- cleanup queue
  await inv(page, "queue_cancel_all");
  await inv(page, "queue_clear_finished");
  for (const j of (await inv(page, "queue_list")).v) await inv(page, "job_remove", { id: j.id });
} catch (e) {
  check("script completed without exception", false, String(e));
} finally {
  try { page?.ws.close(); } catch {}
  try { execFileSync("taskkill", ["/F", "/PID", String(app.pid)], { stdio: "ignore" }); } catch {}
  try { rmSync(work, { recursive: true, force: true }); } catch {}
}
const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length ? 1 : 0);
