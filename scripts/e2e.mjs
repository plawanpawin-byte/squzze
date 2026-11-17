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
          async mouse(type, x, y) {
            await send("Input.dispatchMouseEvent", { type, x, y, button: "left", buttons: type === "mouseReleased" ? 0 : 1, clickCount: 1 });
          },
          async viewport(w, h) {
            await send("Emulation.setDeviceMetricsOverride", { width: w, height: h, deviceScaleFactor: 1, mobile: false });
          },
          async scheme(v) {
            await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: v }] });
          },
          async reload() {
            await send("Page.reload");
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

async function send_key(page, key) {
  await page.eval(`document.activeElement.dispatchEvent(new KeyboardEvent('keydown', {key:${JSON.stringify(key)}, bubbles:true}))`);
}
const inv = (page, cmd, args = {}) =>
  page.eval(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)}).then(v => ({ok:true, v}), e => ({ok:false, e:String(e)}))`);
const text = (page) => page.eval("document.body?.innerText ?? ''");
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
  check("probe shows codec/resolution", t1.includes("h264 · 1280×720"));
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

  // ================= UI behaviour + the user's real recording =================
  const rec = process.env.SQUEEZE_TEST_FILE ?? join(process.env.USERPROFILE ?? "", "Downloads", "Recording 2026-10-22 200301-hevc-1080.mp4");
  const dragPayload = (paths) => `window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {event:'tauri://drag-drop', payload:{paths:${JSON.stringify(paths)}, position:{x:5,y:5}}})`;
  await clickText(page, "[role=tab]", "Current");
  await sleep(300);
  const t0txt = await text(page);
  check("Start batch explains why it is disabled", t0txt.includes("Add a file first") || t0txt.includes("Pick at least"), "hint text present");

  // drag overlay
  await page.eval(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {event:'tauri://drag-enter', payload:{paths:[], position:{x:1,y:1}}})`);
  await sleep(200);
  check("drag-over shows the drop overlay", await page.eval("!!document.querySelector('.dropveil')"));
  await page.eval(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {event:'tauri://drag-leave', payload:null})`);
  await sleep(200);
  check("overlay goes away on leave", !(await page.eval("!!document.querySelector('.dropveil')")));

  // splitters: drag, clamp, persist, reset
  const geo = () => page.eval(`(() => { const q = (s) => document.querySelector(s)?.getBoundingClientRect(); const L = q('.pane'); const S = [...document.querySelectorAll('[role=separator]')].map(e => e.getBoundingClientRect()); return { left: L?.width, seps: S.map(r => ({x: r.x + r.width/2, y: r.y + r.height/2, w: r.width, h: r.height})) } })()`);
  let g = await geo();
  const w0 = g.left;
  const sx = g.seps[0];
  await page.mouse("mousePressed", sx.x, sx.y);
  await page.mouse("mouseMoved", sx.x + 90, sx.y);
  await page.mouse("mouseReleased", sx.x + 90, sx.y);
  await sleep(250);
  g = await geo();
  check("dragging the left splitter widens the sidebar", g.left > w0 + 60, `${w0} -> ${g.left}`);
  const sx2 = g.seps[0];
  await page.mouse("mousePressed", sx2.x, sx2.y);
  await page.mouse("mouseMoved", sx2.x + 900, sx2.y);
  await page.mouse("mouseReleased", sx2.x + 900, sx2.y);
  await sleep(250);
  g = await geo();
  check("sidebar width is clamped to its maximum", g.left <= 441, `width=${g.left}`);
  await page.eval(`(() => { const e=[...document.querySelectorAll('[role=separator]')][0]; e.focus(); })()`);
  await send_key(page, "ArrowLeft");
  await sleep(150);
  const stored = await page.eval(`localStorage.getItem('squeeze.layout.v1')`);
  check("layout is saved", !!stored && JSON.parse(stored).leftW > 170, stored);
  await page.reload();
  await sleep(2500);
  await waitFor(async () => (await text(page)).includes("Squeeze"), 15000);
  g = await geo();
  check("layout is restored after reload", Math.abs(g.left - JSON.parse(stored).leftW) < 2, `${g.left}`);
  await page.eval(`(() => { const e=[...document.querySelectorAll('[role=separator]')][0]; e.dispatchEvent(new MouseEvent('dblclick', {bubbles:true})); })()`);
  await sleep(250);
  g = await geo();
  check("double-click resets the sidebar width", Math.abs(g.left - 250) < 2, `${g.left}`);

  // real recording through the UI with an in/out range
  if (existsSync(rec)) {
    await page.eval(dragPayload([rec]));
    await waitFor(async () => (await text(page)).includes("Recording 2026-10-22"), 20000);
    await sleep(800);
    const tt = await text(page);
    check("real recording probed (hevc 1496×852)", tt.includes("hevc") && tt.includes("1496×852"), "");
    // type In / Out in the inspector
    const setField = (label, val) => page.eval(`(() => { const el=document.querySelector('input[aria-label=${JSON.stringify(label)}]'); el.value=${JSON.stringify(val)}; el.dispatchEvent(new Event('change',{bubbles:true})); return el.value })()`);
    const vin = await setField("In point", "0:05");
    const vout = await setField("Out point", "0:10");
    await sleep(300);
    check("In/Out fields accept mm:ss and normalise", vin.startsWith("00:00:05") && vout.startsWith("00:00:10"), `${vin} / ${vout}`);
    await setField("Out point", "0:03");
    await sleep(200);
    check("out point before in point is rejected", (await page.eval(`document.querySelector('input[aria-label="Out point"]').value`)).startsWith("00:00:10"));
    check("trim range is shown on the file", (await text(page)).includes("Range 00:00:05"), "");
    await page.shot("04-trim.png");
    await clickText(page, ".leaf", "HEVC MP4 — 1080p (smaller");
    await clickText(page, ".leaf", "VP9 WebM — 1080p");
    await sleep(300);
    const outRows = await page.eval("document.querySelectorAll('.source.sel tbody tr').length");
    check("one source gets three outputs", outRows === 3, `rows=${outRows}`);
    const before = (await inv(page, "queue_list")).v.map((j) => j.id);
    await clickText(page, ".primary", "Start batch");
    // observe the slow software VP9 job (AV1 uses NVENC here and finishes too fast to observe)
    await waitFor(async () => (await inv(page, "queue_list")).v.some((j) => !before.includes(j.id) && j.preset.id === "vp9-1080-webm" && j.state === "running" && j.progress > 5), 120000);
    // poll the UI while the job is running: all of encoder, percent and elapsed must be visible at some moment
    const seen = { stats: false, overall: false, fake100: false };
    let last = "";
    await waitFor(async () => {
      last = await text(page);
      seen.stats ||= /Encoding with|Encoding in software/.test(last) && last.includes("elapsed") && /\d+%/.test(last);
      seen.overall ||= /Encoding \d+\/\d+ · \d+%/.test(last);
      if (/Encoding \d+\/\d+ · 100%/.test(last)) {
        const running = (await inv(page, "queue_list")).v.some((j) => j.state === "running" || j.state === "queued");
        seen.fake100 ||= running;
      }
      return seen.stats && seen.overall && last.length > 0 && !(await inv(page, "queue_list")).v.some((j) => j.state === "running" && j.progress < 99);
    }, 15000, 120);
    await page.shot("05-active.png");
    check("Active tab shows encoder, percent and elapsed", seen.stats, last.replace(/\s+/g, " ").slice(200, 520));
    check("overall progress appears in the footer", seen.overall, "");
    check("overall bar never shows 100% while a job is still running", !seen.fake100, "");
    const recJobs = await waitFor(async () => {
      const js = (await inv(page, "queue_list")).v.filter((j) => !before.includes(j.id));
      return js.length === 3 && js.every((j) => j.state === "done" || j.state === "failed") ? js : false;
    }, 180000);
    check("all three outputs of the recording finished", recJobs && recJobs.every((j) => j.state === "done"), recJobs ? recJobs.map((j) => j.preset.id + ":" + j.state).join(",") : "timeout");
    if (recJobs) {
      for (const j of recJobs) {
        check(`${j.preset.id}: verified by ffprobe and 5 s long`, j.verify?.ok && Math.abs((j.verify.duration_secs ?? 0) - 5) < 0.4, JSON.stringify(j.verify?.notes ?? []) + ` dur=${j.verify?.duration_secs}`);
        check(`${j.preset.id}: command line and log recorded`, (j.cmdline ?? "").includes("-ss 5.000") && (j.cmdline ?? "").includes("-t 5.000") && j.log.length > 0, (j.cmdline ?? "").slice(0, 60));
        check(`${j.preset.id}: timing recorded`, j.started_ms && j.finished_ms && j.finished_ms >= j.started_ms, "");
        const pj = probeJson(j.output);
        check(`${j.preset.id}: independent ffprobe agrees`, Math.abs(Number(pj.format.duration) - 5) < 0.4 && pj.streams.some((s) => s.codec_type === "video"), j.output);
      }
      await clickText(page, "[role=tab]", "Completed");
      await sleep(500);
      const ct = await text(page);
      check("Completed tab shows ✓ Verified and the range size (no misleading % for trimmed jobs)", ct.includes("✓ Verified") && ct.includes("for the selected range") && !/\(−\d+%\)/.test(ct.split("Range")[1] ?? ""), "");
      await clickText(page, ".acts .btn", "Details");
      await sleep(300);
      const det = await page.eval("document.querySelector('pre.details')?.innerText ?? ''");
      check("Details shows command line and FFmpeg output", det.includes("Command:") && det.includes("ffmpeg ") && det.includes("Verified:"), det.slice(0, 40));
      await page.shot("06-completed.png");
      for (const j of recJobs) {
        try { rmSync(j.output, { force: true }); } catch {}
      }
    }
  } else {
    console.log("SKIP: user recording not found at", rec);
  }

  // ---- filename template + overwrite policy from the UI
  await clickText(page, "[role=tab]", "Current");
  await clickText(page, ".side [role=tab]", "Locations");
  await sleep(300);
  const setInput = (label, val) => page.eval(`(() => { const el=document.querySelector('[aria-label=${JSON.stringify(label)}]'); el.value=${JSON.stringify(val)}; el.dispatchEvent(new Event('change',{bubbles:true})); return el.value })()`);
  await setInput("Output name template", "OUT_{name}_{preset}");
  await setInput("If the output file already exists", "skip");
  await sleep(500);
  const st = (await inv(page, "get_settings")).v;
  check("template and overwrite policy are saved from the UI", st.template === "OUT_{name}_{preset}" && st.overwrite === "skip", JSON.stringify(st));
  const runWav = async () => {
    const known = (await inv(page, "queue_list")).v.map((j) => j.id);
    await page.eval(dragPayload([wav]));
    await waitFor(async () => (await text(page)).includes("tone.wav"), 15000);
    await sleep(500);
    await clickText(page, ".primary", "Start batch");
    return waitFor(async () => {
      const js = (await inv(page, "queue_list")).v.filter((j) => !known.includes(j.id));
      return js.length && js.every((j) => j.state === "done" || j.state === "failed") ? js[0] : false;
    }, 60000);
  };
  const first = await runWav();
  check("custom name template is used", first && first.state === "done" && first.output.endsWith("OUT_tone_aac-256.m4a"), first?.output);
  const second = await runWav();
  check("'skip' policy fails the job instead of overwriting", second && second.state === "failed" && /already exists/.test(second.error ?? ""), second?.error);
  await setInput("If the output file already exists", "rename");
  await sleep(400);
  const third = await runWav();
  check("'keep both' adds a number", third && third.state === "done" && third.output.endsWith("OUT_tone_aac-256 (1).m4a"), third?.output);
  for (const j of [first, third]) { try { rmSync(j.output, { force: true }); } catch {} }
  await setInput("Output name template", "{name}-{preset}");
  await clickText(page, ".side [role=tab]", "Presets");
  const ffv = await page.eval(`document.querySelector('.ffv')?.innerText ?? ''`);
  check("footer shows the FFmpeg version", /FFmpeg \d/.test(ffv), ffv);

  // ---- Active tab after a finished run + responsive + themes
  await clickText(page, "[role=tab]", "Active");
  await sleep(400);
  const at2 = await text(page);
  check("Active tab says the batch finished and links to results", at2.includes("Batch finished") && at2.includes("View results"), "");
  await clickText(page, "[role=tab]", "Current");
  for (const [w, h] of [[1000, 640], [960, 600], [1440, 900]]) {
    await page.viewport(w, h);
    await sleep(500);
    const ov = await page.eval(`({ sw: document.documentElement.scrollWidth, iw: window.innerWidth, cw: document.querySelector('.center').getBoundingClientRect().width })`);
    check(`no horizontal overflow and a usable centre at ${w}×${h}`, ov.sw <= ov.iw + 1 && ov.cw >= 340, JSON.stringify(ov));
  }
  const col = (sel) => page.eval(`getComputedStyle(document.querySelector(${JSON.stringify(sel)})).backgroundColor`);
  const darkBg = await col(".top");
  await page.scheme("light");
  await sleep(300);
  const lightBg = await col(".top");
  await page.scheme("dark");
  await sleep(200);
  check("light and dark themes both apply", darkBg !== lightBg && lightBg === "rgb(246, 246, 246)", `${darkBg} / ${lightBg}`);

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
