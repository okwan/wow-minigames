// shoot.mjs - drive the served wenilla client in headless Chromium (CPU WebGPU via SwiftShader),
// record every /data/ request, and take screenshots of the Warcraft Survivors autopilot run.
//
//   PW_DIR=<dir with `npm i playwright`> node scripts/ci/shoot.mjs [outDir]
//
// Env: BASE_URL (default http://127.0.0.1:8090), SURVIVORS_CLASS (Mage), SURVIVORS_MAP (Elwynn),
//      READY_TIMEOUT_S (900), SHOT_TIMES_S ("60,120,240").
//
// Query keys (benilla_app::webenv: WOW_<NAME> -> ?<name> lowercased): WOW_SURVIVORS_AUTO ->
// ?survivors_auto=<class>, WOW_SURVIVORS_MAP -> ?survivors_map=<name|index>.
//
// This script never throws out of main(): a missing screenshot is a FAIL line in RESULT.md, not
// a failed workflow (the caller also runs it continue-on-error).
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const OUT = path.resolve(process.argv[2] || 'ci-out');
const BASE = process.env.BASE_URL || 'http://127.0.0.1:8090';
const CLASS = process.env.SURVIVORS_CLASS || 'Mage';
const MAP = process.env.SURVIVORS_MAP || 'Elwynn';
const READY_TIMEOUT_MS = Number(process.env.READY_TIMEOUT_S || 900) * 1000;
const SHOT_TIMES_S = (process.env.SHOT_TIMES_S || '60,120,240').split(',').map(Number);
const URL_ = `${BASE}/?survivors_auto=${encodeURIComponent(CLASS)}&survivors_map=${encodeURIComponent(MAP)}`;
const GB = 1000 ** 3, MB = 1000 ** 2;

fs.mkdirSync(OUT, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const t0 = Date.now();
const since = () => ((Date.now() - t0) / 1000).toFixed(1);

const consoleLines = [];
const logConsole = (s) => { consoleLines.push(`[${since()}s] ${s}`); };
const note = (s) => { console.log(`[shoot ${since()}s] ${s}`); logConsole(`(shoot) ${s}`); };

// Flag sets, tried in order; the first one where navigator.gpu yields an adapter wins.
const COMMON = ['--ignore-gpu-blocklist', '--enable-webgpu-developer-features', '--no-sandbox'];
const FLAG_SETS = [
  { name: 'swiftshader-vulkan', args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan', '--use-angle=swiftshader', '--use-gl=angle', ...COMMON] },
  { name: 'swiftshader-vulkan-use-vulkan', args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan', '--use-vulkan=swiftshader', '--use-angle=swiftshader', '--use-gl=angle', '--disable-vulkan-surface', ...COMMON] },
  { name: 'angle-swiftshader-webgpu-only', args: ['--enable-unsafe-webgpu', '--use-webgpu-adapter=swiftshader', '--use-angle=swiftshader', ...COMMON] },
  { name: 'minimal-unsafe-webgpu', args: ['--enable-unsafe-webgpu', ...COMMON] },
];

// ---- /data/ request ledger ------------------------------------------------------------------
const dataFiles = new Map(); // url -> { name, bytes, wire, status }
let dataRequestCount = 0;
function attachLedger(page) {
  page.on('response', async (resp) => {
    const url = resp.url();
    const i = url.indexOf('/data/');
    if (i < 0) return;
    dataRequestCount++;
    let name = url.slice(i + 6).split('?')[0];
    try { name = decodeURIComponent(name); } catch { /* keep raw */ }
    let wire = Number(resp.headers()['content-length']);
    let bytes = Number.isFinite(wire) && wire > 0 ? wire : null;
    if (bytes === null) {
      try { bytes = (await resp.body()).length; } catch { bytes = 0; }
    }
    const prev = dataFiles.get(url);
    dataFiles.set(url, { name, bytes: Math.max(bytes, prev?.bytes || 0), status: resp.status() });
  });
}

function dataSummary() {
  const files = [...dataFiles.values()];
  const byExt = {};
  let total = 0, largest = null, over100 = 0;
  for (const f of files) {
    total += f.bytes;
    const m = /\.([A-Za-z0-9_]+)$/.exec(f.name);
    const ext = m ? m[1].toLowerCase() : '(none)';
    byExt[ext] ||= { files: 0, bytes: 0 };
    byExt[ext].files++; byExt[ext].bytes += f.bytes;
    if (!largest || f.bytes > largest.bytes) largest = { name: f.name, bytes: f.bytes };
    if (f.bytes > 100 * MB) over100++;
  }
  return {
    note: 'bytes = content-length when present (wire size, may be compressed) else decoded body length; unique URLs counted once for the tree-size figures',
    request_count: dataRequestCount,
    unique_files: files.length,
    total_bytes: total,
    per_extension: byExt,
    largest_file: largest,
    tree_size: { total_bytes: total, limit_bytes: GB, fraction_of_1gb_limit: +(total / GB).toFixed(4), files_over_100mb: over100 },
    names: files.map((f) => f.name).sort(),
  };
}
function writeLedger() {
  fs.writeFileSync(path.join(OUT, 'data-requests.json'), JSON.stringify(dataSummary(), null, 2));
  fs.writeFileSync(path.join(OUT, 'console.log'), consoleLines.join('\n') + '\n');
}

// ---- pixel check ---------------------------------------------------------------------------
// Decode a PNG screenshot in a throwaway page's canvas: fraction of non-dark pixels and the
// number of distinct coarse colours. "World rendered" = both clearly above a black/blank frame.
async function frameStats(helper, png) {
  return helper.evaluate(async (b64) => {
    const img = new Image();
    img.src = 'data:image/png;base64,' + b64;
    await img.decode();
    const w = 160, h = 90;
    const c = document.createElement('canvas'); c.width = w; c.height = h;
    const g = c.getContext('2d', { willReadFrequently: true });
    g.drawImage(img, 0, 0, w, h);
    const d = g.getImageData(0, 0, w, h).data;
    let lit = 0; const buckets = new Set();
    for (let i = 0; i < d.length; i += 4) {
      if (d[i] + d[i + 1] + d[i + 2] > 60) lit++;
      buckets.add(((d[i] >> 5) << 6) | ((d[i + 1] >> 5) << 3) | (d[i + 2] >> 5));
    }
    return { lit_fraction: lit / (w * h), colours: buckets.size };
  }, png.toString('base64'));
}
const looksRendered = (s) => s.lit_fraction > 0.15 && s.colours >= 12;

// ---- main ----------------------------------------------------------------------------------
const result = { status: 'FAIL', reason: 'did not start', flagSet: null, shots: [], rendered: false, runBegan: false };

async function main() {
  const pwDir = process.env.PW_DIR;
  if (!pwDir) throw new Error('PW_DIR (directory where `npm i playwright` ran) is not set');
  const { chromium } = createRequire(path.join(path.resolve(pwDir), 'x.js'))('playwright');

  let browser, page, helper;
  for (const fs_ of FLAG_SETS) {
    note(`trying flag set "${fs_.name}": ${fs_.args.join(' ')}`);
    try {
      browser = await chromium.launch({ headless: true, args: fs_.args });
      const probe = await browser.newPage();
      await probe.goto('about:blank');
      const info = await probe.evaluate(async () => {
        if (!navigator.gpu) return { ok: false, why: 'no navigator.gpu' };
        const a = await navigator.gpu.requestAdapter();
        if (!a) return { ok: false, why: 'requestAdapter() returned null' };
        let desc = {};
        try { desc = a.info ? { vendor: a.info.vendor, architecture: a.info.architecture, description: a.info.description } : {}; } catch {}
        try { await a.requestDevice(); } catch (e) { return { ok: false, why: 'requestDevice: ' + e }; }
        return { ok: true, desc };
      });
      await probe.close();
      note(`  probe: ${JSON.stringify(info)}`);
      if (info.ok) { result.flagSet = fs_.name; break; }
    } catch (e) { note(`  launch/probe failed: ${e.message}`); }
    try { await browser?.close(); } catch {}
    browser = null;
  }
  if (!browser) { result.reason = 'no Chromium flag set exposed a WebGPU adapter (see console.log)'; return; }
  note(`WebGPU works with flag set: ${result.flagSet}`);

  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  page = await ctx.newPage();
  helper = await ctx.newPage();
  await helper.goto('about:blank');
  attachLedger(page);
  page.on('console', (m) => logConsole(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logConsole(`pageerror: ${e.message}`));
  page.on('crash', () => logConsole('PAGE CRASHED'));
  page.on('requestfailed', (r) => { if (r.url().includes('/data/')) logConsole(`requestfailed: ${r.url()} ${r.failure()?.errorText}`); });

  note(`open ${URL_}`);
  await page.goto(URL_, { waitUntil: 'commit', timeout: 120000 });

  const isBegun = () => consoleLines.some((l) => l.includes('survivors auto: run begins'));
  const sawCapture = () => consoleLines.some((l) => l.includes('survivors auto: capture'));
  let anchor = null; // ms timestamp at which the character is judged visible
  let lastPoll = 0;
  while (Date.now() - t0 < READY_TIMEOUT_MS) {
    await sleep(5000);
    if (page.isClosed()) { result.reason = 'page closed'; return; }
    if (isBegun()) result.runBegan = true;
    if (Date.now() - lastPoll < 15000) continue; // pixel-check every 15 s only (screenshots are costly on CPU)
    lastPoll = Date.now();
    let stats;
    try { stats = await frameStats(helper, await page.screenshot({ type: 'png', timeout: 60000 })); }
    catch (e) { note(`screenshot probe failed: ${e.message}`); continue; }
    note(`probe: began=${result.runBegan} capture=${sawCapture()} lit=${stats.lit_fraction.toFixed(2)} colours=${stats.colours}`);
    if (result.runBegan && looksRendered(stats)) { anchor = Date.now(); result.rendered = true; break; }
  }
  if (!anchor) {
    // Last resort: a non-black frame without the log hint still gets screenshotted.
    try {
      const png = await page.screenshot({ type: 'png', timeout: 60000 });
      const stats = await frameStats(helper, png);
      fs.writeFileSync(path.join(OUT, 'shot-timeout.png'), png);
      result.reason = `timeout after ${READY_TIMEOUT_MS / 1000}s waiting for the run to begin and the canvas to render (runBegan=${result.runBegan}, last frame lit=${stats.lit_fraction.toFixed(2)} colours=${stats.colours}); see shot-timeout.png and console.log`;
    } catch (e) { result.reason = `timeout waiting for the world; final screenshot failed: ${e.message}`; }
    return;
  }
  note(`world rendered; character judged visible — taking shots at +${SHOT_TIMES_S.join('s, +')}s`);

  let n = 0;
  for (const at of SHOT_TIMES_S) {
    const wait = anchor + at * 1000 - Date.now();
    if (wait > 0) await sleep(wait);
    n++;
    try {
      const png = await page.screenshot({ type: 'png', timeout: 120000 });
      const stats = await frameStats(helper, png);
      const file = `shot-${n}.png`;
      fs.writeFileSync(path.join(OUT, file), png);
      result.shots.push({ file, at_s: at, ...stats });
      note(`${file}: lit=${stats.lit_fraction.toFixed(2)} colours=${stats.colours}`);
    } catch (e) { note(`shot ${n} failed: ${e.message}`); }
  }
  if (result.shots.length === SHOT_TIMES_S.length) { result.status = 'PASS'; result.reason = 'world rendered; all screenshots taken'; }
  else if (result.shots.length) { result.status = 'FAIL'; result.reason = `only ${result.shots.length}/${SHOT_TIMES_S.length} screenshots taken`; }
  else { result.reason = 'world rendered but every screenshot failed'; }
  await browser.close();
}

try { await main(); }
catch (e) { result.reason = `script error: ${e && e.stack || e}`; note(result.reason); }

writeLedger();
const d = dataSummary();
const md = [
  `# wenilla CI screenshot run: ${result.status}`,
  '',
  `- reason: ${result.reason}`,
  `- url: \`${URL_}\``,
  `- Chromium flag set that exposed WebGPU: ${result.flagSet ?? 'none'}`,
  `- run began (console hint): ${result.runBegan}; canvas rendered (pixel check): ${result.rendered}`,
  `- screenshots: ${result.shots.map((s) => `${s.file} (+${s.at_s}s, lit ${s.lit_fraction.toFixed(2)}, ${s.colours} colours)`).join('; ') || 'none'}`,
  '',
  '## /data/ tree touched during this run',
  `- requests: ${d.request_count}, unique files: ${d.unique_files}`,
  `- total: ${(d.total_bytes / MB).toFixed(1)} MB = ${(d.tree_size.fraction_of_1gb_limit * 100).toFixed(2)}% of the 1 GB Pages site limit`,
  `- files over 100 MB: ${d.tree_size.files_over_100mb}`,
  `- largest: ${d.largest_file ? `${d.largest_file.name} (${(d.largest_file.bytes / MB).toFixed(1)} MB)` : 'n/a'}`,
  '',
  '(Informational only: the Pages deploy ships no game data. This measures what a data-bearing host would have to serve.)',
  '',
].join('\n');
fs.writeFileSync(path.join(OUT, 'RESULT.md'), md);
console.log(md);
process.exit(0);
