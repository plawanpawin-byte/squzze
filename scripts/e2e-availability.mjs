// Debug-build E2E: with some encoders hidden, the UI and backend must refuse the affected presets clearly.
// Usage: node scripts/e2e-availability.mjs <debug squeeze.exe>   (the debug build needs the Vite dev server on :1420)
import { spawn, execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const exe = process.argv[2];
const PORT = 9337, sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (n, ok, d = "") => { results.push(ok); console.log(`${ok ? "PASS" : "FAIL"}  ${n}${d ? "  — " + d : ""}`); };
const work = mkdtempSync(join(tmpdir(), "sq-avail-"));
const clip = join(work, "clip.mp4");
execFileSync("ffmpeg", ["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=30:duration=2", "-f", "lavfi", "-i", "sine=f=440:duration=2", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest", clip]);
const app = spawn(exe, [], { env: { ...process.env, SQUEEZE_TEST_HIDE_ENCODERS: "libsvtav1,av1_nvenc,libx265,hevc_nvenc,hevc_amf", WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` }, stdio: "ignore" });
let ws, id = 0; const pend = new Map();
try {
  for (let i = 0; i < 60 && !ws; i++) { try { const p = (await (await fetch(`http://127.0.0.1:${PORT}/json`)).json()).find((t) => t.type === "page"); if (p) { ws = new WebSocket(p.webSocketDebuggerUrl); await new Promise((r) => (ws.onopen = r)); } } catch {} if (!ws) await sleep(500); }
  ws.onmessage = (m) => { const d = JSON.parse(m.data); pend.get(d.id)?.(d); };
  const send = (method, params = {}) => new Promise((r) => { const i = ++id; pend.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  const ev = async (e) => (await send("Runtime.evaluate", { expression: e, awaitPromise: true, returnByValue: true })).result?.result?.value;
  const inv = (c, a = {}) => ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(c)}, ${JSON.stringify(a)}).then(v=>({ok:true,v}),e=>({ok:false,e:String(e)}))`);
  await sleep(3000);
  const av = (await inv("preset_availability")).v;
  check("hidden encoders make the matching presets unavailable", av["av1-1080-webm"] && av["hevc-1080"] && av["hevc-1080-small"], Object.keys(av).join(","));
  check("presets with working encoders stay available", !av["h264-1080"] && !av["vp9-1080-webm"] && !av["aac-256"], "");
  check("the reason names the missing encoder and a fix", /libsvtav1/.test(av["av1-1080-webm"]) && /winget/.test(av["av1-1080-webm"]), av["av1-1080-webm"]);
  const rej = await inv("add_job", { source: clip, presetId: "av1-1080-webm", destDir: null, startSecs: null, endSecs: null });
  check("backend refuses to queue an unavailable preset", !rej.ok && /libsvtav1/.test(rej.e), rej.e);
  const okJob = await inv("add_job", { source: clip, presetId: "h264-1080", destDir: join(work, "out"), startSecs: null, endSecs: null });
  check("an available preset still queues", okJob.ok, okJob.e);
  await inv("queue_cancel_all");
  // UI
  await ev(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit',{event:'tauri://drag-drop',payload:{paths:${JSON.stringify([clip])},position:{x:1,y:1}}})`);
  await sleep(3000);
  const ui = await ev(`(() => { const l=[...document.querySelectorAll('.leaf')].find(e=>e.innerText.includes('AV1 WebM')); return l ? { off: l.classList.contains('off'), text: l.innerText, title: l.title } : null })()`);
  check("sidebar marks the preset 'not available' with the reason as tooltip", ui && ui.off && /not available/.test(ui.text) && /libsvtav1/.test(ui.title), JSON.stringify(ui));
  await ev(`[...document.querySelectorAll('.leaf')].find(e=>e.innerText.includes('AV1 WebM')).click()`);
  await sleep(400);
  const msg = await ev(`document.querySelector('.msg')?.innerText ?? ''`);
  check("clicking it shows the reason instead of adding an output", /libsvtav1/.test(msg), msg);
  const rows = await ev(`document.querySelectorAll('.source.sel tbody tr').length`);
  check("no output row was added", rows === 1, `rows=${rows}`);
  const opt = await ev(`(() => { const o=[...document.querySelectorAll('.source select option')].find(o=>o.textContent.includes('AV1 WebM')); return o ? { disabled: o.disabled, text: o.textContent } : null })()`);
  check("the preset dropdown disables it too", opt && opt.disabled && /not available/.test(opt.text), JSON.stringify(opt));
  // the default preset for a new file must be an available one
  const defRow = await ev(`document.querySelector('.source.sel tbody select')?.value`);
  check("default output preset is an available one", defRow === "h264-1080", defRow);
} catch (e) {
  check("script completed", false, String(e));
} finally {
  try { ws?.close(); } catch {}
  try { execFileSync("taskkill", ["/F", "/PID", String(app.pid)], { stdio: "ignore" }); } catch {}
  try { rmSync(work, { recursive: true, force: true }); } catch {}
}
const bad = results.filter((r) => !r).length;
console.log(`\n${results.length - bad}/${results.length} checks passed`);
process.exit(bad ? 1 : 0);
