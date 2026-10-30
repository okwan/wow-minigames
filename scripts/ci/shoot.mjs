// shoot.mjs - drive the served wenilla client in headless Chromium (CPU WebGPU via SwiftShader),
// record every /data/ request, and take screenshots of the sandbox (the character standing in the world).
//
//   PW_DIR=<dir with `npm i playwright`> node scripts/ci/shoot.mjs [outDir]
//
// Env: BASE_URL (default http://127.0.0.1:8090), SANDBOX_RACE (human), SANDBOX_CLASS (mage),
//      SANDBOX_QUERY (extra query string, e.g. "map=1&x=-950&y=-2700"), READY_TIMEOUT_S (900),
//      SHOT_TIMES_S ("60,120,240").
//
// Query keys (benilla_app::sandbox): ?race= ?class= ?gender= ?map= ?x= ?y= ?z= ?facing= ?time=.
//
// This script never throws out of main(): a missing screenshot is a FAIL line in RESULT.md, not
// a failed workflow (the caller also runs it continue-on-error).
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const OUT = path.resolve(process.argv[2] || 'ci-out');
const BASE = process.env.BASE_URL || 'http://127.0.0.1:8090';
const RACE = process.env.SANDBOX_RACE || 'human';
const CLASS = process.env.SANDBOX_CLASS || 'mage';
const EXTRA = process.env.SANDBOX_QUERY ? `&${process.env.SANDBOX_QUERY}` : '';
const READY_TIMEOUT_MS = Number(process.env.READY_TIMEOUT_S || 900) * 1000;
const SHOT_TIMES_S = (process.env.SHOT_TIMES_S || '60,120,240').split(',').map(Number);
const URL_ = `${BASE}/?race=${encodeURIComponent(RACE)}&class=${encodeURIComponent(CLASS)}${EXTRA}`;
const GB = 1000 ** 3, MB = 1000 ** 2;

fs.mkdirSync(OUT, { recursive: true });
const withTimeout = (p, ms, what) => Promise.race([p, new Promise((_, rj) => setTimeout(() => rj(new Error(`${what} hung > ${ms / 1000}s`)), ms))]);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const t0 = Date.now();
const since = () => ((Date.now() - t0) / 1000).toFixed(1);

const consoleLines = [];
let suppressed = 0;
const NOISE = /Failed to load resource|404|requestfailed|ERR_FAILED|net::/i;
const logConsole = (s) => { if (NOISE.test(s) && !/panic|unreachable|Validation|Destroyed/i.test(s)) { suppressed++; return; } consoleLines.push(`[${since()}s] ${s}`); };
const note = (s) => { console.log(`[shoot ${since()}s] ${s}`); logConsole(`(shoot) ${s}`); };

// Launch configs, tried in order; the first one where navigator.gpu yields an adapter AND a device
// wins. `channel: 'chromium'` selects the new headless mode (the old headless shell has no
// WebGPU). Headed sets need a display (the workflow runs this under xvfb-run).
const COMMON = ['--ignore-gpu-blocklist', '--enable-webgpu-developer-features', '--no-sandbox',
  '--disable-gpu-vsync', '--disable-frame-rate-limit', '--run-all-compositor-stages-before-draw',
  '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'];
const LVP_ICD = '/usr/share/vulkan/icd.d/lvp_icd.x86_64.json';
const SWS_VK = ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=vulkan', '--use-vulkan=swiftshader',
  '--use-webgpu-adapter=swiftshader', '--enable-unsafe-swiftshader', '--disable-vulkan-surface', ...COMMON];
const FLAG_SETS = [
  { name: 'a-new-headless-swiftshader-vulkan', opts: { channel: 'chromium', headless: true }, args: SWS_VK },
  { name: 'b-headed-xvfb-swiftshader-vulkan', opts: { channel: 'chromium', headless: false }, args: SWS_VK, needsDisplay: true },
  { name: 'c-new-headless-angle-swiftshader-gl', opts: { channel: 'chromium', headless: true },
    args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=swiftshader', '--use-gl=angle', '--enable-unsafe-swiftshader', ...COMMON] },
  { name: 'd-new-headless-lavapipe', opts: { channel: 'chromium', headless: true }, env: { VK_ICD_FILENAMES: LVP_ICD },
    args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=vulkan', '--use-vulkan=native', '--disable-vulkan-surface', ...COMMON] },
  { name: 'e-headed-xvfb-lavapipe', opts: { channel: 'chromium', headless: false }, env: { VK_ICD_FILENAMES: LVP_ICD }, needsDisplay: true,
    args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=vulkan', '--use-vulkan=native', '--disable-vulkan-surface', ...COMMON] },
  { name: 'f-old-headless-shell-minimal', opts: { headless: true }, args: ['--enable-unsafe-webgpu', ...COMMON] },
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
  const ds = dataSummary();
  fs.writeFileSync(path.join(OUT, 'data-requests.json'), JSON.stringify(ds, null, 2));
  fs.writeFileSync(path.join(OUT, 'data-names.txt'), ds.names.join('\n') + (ds.names.length ? '\n' : ''));
  fs.writeFileSync(path.join(OUT, 'console.log'), consoleLines.join('\n') + `\n(suppressed ${suppressed} 404/failed-resource lines)\n`);
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
const looksRendered = (s) => s.lit_fraction > 0.05 && s.colours >= 12;

// ---- main ----------------------------------------------------------------------------------
let finished = false;
const result = { status: 'FAIL', reason: 'did not start', flagSet: null, shots: [], rendered: false, runBegan: false };

async function main() {
  const pwDir = process.env.PW_DIR;
  if (!pwDir) throw new Error('PW_DIR (directory where `npm i playwright` ran) is not set');
  const { chromium } = createRequire(path.join(path.resolve(pwDir), 'x.js'))('playwright');

  let browser, page, helper;
  let gpuDumped = false;
  for (const fs_ of FLAG_SETS) {
    if (fs_.needsDisplay && !process.env.DISPLAY) { note(`skip "${fs_.name}": no DISPLAY (run under xvfb-run)`); continue; }
    note(`trying "${fs_.name}": ${JSON.stringify(fs_.opts)} ${fs_.env ? JSON.stringify(fs_.env) : ''} ${fs_.args.join(' ')}`);
    try {
      browser = await chromium.launch({ ...fs_.opts, args: fs_.args, env: { ...process.env, ...(fs_.env || {}) } });
      note(`  browser version: ${browser.version()}`);
      const probe = await browser.newPage();
      // Same origin as the game: 127.0.0.1 is a secure context, so navigator.gpu is exposed.
      await probe.route('**/__gpu_probe__', (route) => route.fulfill({ status: 200, contentType: 'text/html', body: '<!doctype html><title>probe</title>' }));
      await probe.goto(`${BASE}/__gpu_probe__`).catch((e) => note(`  probe goto: ${e.message}`));
      const info = await probe.evaluate(async () => {
        const base = { hasGpu: !!navigator.gpu, isSecureContext: window.isSecureContext, ua: navigator.userAgent };
        if (!navigator.gpu) return { ok: false, why: 'no navigator.gpu', ...base };
        const a = await navigator.gpu.requestAdapter();
        if (!a) return { ok: false, why: 'requestAdapter() returned null', ...base };
        let desc = {};
        try { desc = a.info ? { vendor: a.info.vendor, architecture: a.info.architecture, device: a.info.device, description: a.info.description } : {}; } catch {}
        const limits = { maxStorageBuffersPerShaderStage: a.limits?.maxStorageBuffersPerShaderStage, maxBufferSize: a.limits?.maxBufferSize };
        try { await a.requestDevice(); } catch (e) { return { ok: false, why: 'requestDevice: ' + e, desc, ...base }; }
        return { ok: true, desc, limits, features: [...a.features], ...base };
      });
      note(`  probe: ${JSON.stringify(info)}`);
      if (!gpuDumped || !info.ok) {
        try {
          const g = await browser.newPage();
          await g.goto('chrome://gpu', { timeout: 30000 });
          await sleep(3000);
          const txt = await g.evaluate(() => document.body.innerText);
          fs.appendFileSync(path.join(OUT, 'gpu.txt'), `\n===== chrome://gpu for "${fs_.name}" (ok=${info.ok}) =====\n${txt}\n`);
          gpuDumped = true;
          await g.close();
        } catch (e) { note(`  chrome://gpu dump failed: ${e.message}`); }
      }
      await probe.close();
      if (info.ok) { result.flagSet = fs_.name; break; }
    } catch (e) { note(`  launch/probe failed: ${e.message}`); }
    try { await browser?.close(); } catch {}
    browser = null;
  }
  if (!browser) { result.reason = 'no Chromium flag set exposed a WebGPU adapter (see console.log)'; return; }
  note(`WebGPU works with flag set: ${result.flagSet}`);

  // ---- attempts: each gets a budget, then we move on instead of waiting out one for 900 s -------
  const ATTEMPTS = [
    { name: 'full', query: '' },
    { name: 'reduced', query: '&renderScale=0.5&farclip=120&worlddetail=0' },
  ];
  const BEGIN_BUDGET_MS = Number(process.env.BEGIN_BUDGET_S || 420) * 1000; // to "run begins"
  const RENDER_BUDGET_MS = Number(process.env.RENDER_BUDGET_S || 360) * 1000; // after "run begins"
  const KEY_ERR = /panic|unreachable|Validation|lost|Destroyed/i;
  const saveLoading = (png) => { if (!fs.existsSync(path.join(OUT, 'shot-loading.png'))) fs.writeFileSync(path.join(OUT, 'shot-loading.png'), png); };

  for (const att of ATTEMPTS) {
    const url = URL_ + att.query;
    note(`=== attempt "${att.name}": ${url}`);
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1 });
    page = await ctx.newPage();
    helper = await ctx.newPage();
    await helper.goto('about:blank');
    attachLedger(page);
    const mark = consoleLines.length;
    page.on('console', (m) => {
      const t = m.text();
      logConsole(`${m.type()}: ${t}`);
      if (KEY_ERR.test(t)) note(`KEY console ${m.type()}: ${t.slice(0, 300)}`);
    });
    page.on('pageerror', (e) => { logConsole(`pageerror: ${e.message}`); note(`KEY pageerror: ${e.message.slice(0, 300)}`); });
    page.on('crash', () => { logConsole('PAGE CRASHED'); note('PAGE CRASHED'); });
    page.on('requestfailed', (r) => { if (r.url().includes('/data/')) logConsole(`requestfailed: ${r.url()} ${r.failure()?.errorText}`); });
    let cdp = null;
    try { cdp = await ctx.newCDPSession(page); } catch (e) { note(`no CDP session: ${e.message}`); }

    // Grab the frame several ways; blank compositor shots of a WebGPU canvas are a known failure.
    const grab = async () => {
      await page.bringToFront().catch(() => {});
      const cands = [];
      try { cands.push({ mode: 'page.screenshot', png: await withTimeout(page.screenshot({ type: 'png', timeout: 25000 }), 25000, 'page.screenshot') }); } catch (e) { note(`page.screenshot failed: ${e.message.split('\n')[0]}`); }
      if (cdp) {
        try {
          const r = await withTimeout(cdp.send('Page.captureScreenshot', { format: 'png', fromSurface: true }), 25000, 'cdp capture');
          cands.push({ mode: 'cdp.fromSurface', png: Buffer.from(r.data, 'base64') });
        } catch (e) { note(`cdp capture failed: ${e.message.split('\n')[0]}`); }
      }
      try {
        const d = await withTimeout(page.evaluate(() => {
          const c = document.querySelector('canvas'); if (!c) return null;
          try {
            const o = document.createElement('canvas'); o.width = c.width; o.height = c.height;
            o.getContext('2d').drawImage(c, 0, 0);
            return o.toDataURL('image/png');
          } catch (e) { return null; }
        }), 25000, 'canvas readback');
        if (d) cands.push({ mode: 'canvas.readback', png: Buffer.from(d.split(',')[1], 'base64') });
      } catch (e) { note(`canvas readback failed: ${e.message.split('\n')[0]}`); }
      let best = null;
      for (const c of cands) {
        try { c.stats = await withTimeout(frameStats(helper, c.png), 25000, 'frameStats'); } catch { continue; }
        if (!best || c.stats.lit_fraction * c.stats.colours > best.stats.lit_fraction * best.stats.colours) best = c;
      }
      if (best) note(`grab: ${cands.map((c) => `${c.mode}=${c.stats ? c.stats.lit_fraction.toFixed(2) + '/' + c.stats.colours : 'err'}`).join(' ')} best=${best.mode}`);
      return best;
    };

    const isBegun = () => consoleLines.slice(mark).some((l) => l.includes('sandbox: character spawned'));
    const tStart = Date.now();
    let tBegan = null, anchor = null, bestBlank = null;
    try { await page.goto(url, { waitUntil: 'commit', timeout: 120000 }); } catch (e) { note(`goto: ${e.message.split('\n')[0]}`); }
    while (true) {
      const now = Date.now();
      if (now - t0 > READY_TIMEOUT_MS * 2) break;
      if (!tBegan && now - tStart > BEGIN_BUDGET_MS) { note(`attempt "${att.name}": run never began within ${BEGIN_BUDGET_MS / 1000}s`); break; }
      if (tBegan && now - tBegan > RENDER_BUDGET_MS) { note(`attempt "${att.name}": nothing rendered ${RENDER_BUDGET_MS / 1000}s after run began`); break; }
      await sleep(5000);
      if (page.isClosed()) { note('page closed'); break; }
      if (isBegun()) { result.runBegan = true; tBegan ||= Date.now(); }
      const g = await grab();
      if (!g) continue;
      note(`probe[${att.name}]: began=${!!tBegan} lit=${g.stats.lit_fraction.toFixed(2)} colours=${g.stats.colours}`);
      if (!bestBlank || g.stats.colours > bestBlank.stats.colours) bestBlank = g;
      if (g.stats.colours >= 8 && g.stats.lit_fraction > 0.02 && !tBegan) saveLoading(g.png);
      if (tBegan && looksRendered(g.stats)) { anchor = Date.now(); result.rendered = true; result.flagSet = `${result.flagSet} + ${att.name}${g.mode ? ' (' + g.mode + ')' : ''}`; break; }
    }
    if (!anchor) {
      if (bestBlank) { fs.writeFileSync(path.join(OUT, `shot-timeout-${att.name}.png`), bestBlank.png); }
      result.reason = `attempt "${att.name}" never rendered the world (runBegan=${result.runBegan}); see shot-timeout-*.png, console.log`;
      await ctx.close().catch(() => {});
      continue;
    }
    note(`world rendered; taking shots at +${SHOT_TIMES_S.join('s, +')}s`);
    let n = 0;
    for (const at of SHOT_TIMES_S) {
      const wait = anchor + at * 1000 - Date.now();
      if (wait > 0) await sleep(wait);
      n++;
      try {
        const g = await grab();
        if (!g) throw new Error('no capture');
        const file = `shot-${n}.png`;
        fs.writeFileSync(path.join(OUT, file), g.png);
        result.shots.push({ file, at_s: at, ...g.stats, good: looksRendered(g.stats) });
        note(`${file}: ${g.mode} lit=${g.stats.lit_fraction.toFixed(2)} colours=${g.stats.colours}`);
      } catch (e) { note(`shot ${n} failed: ${e.message}`); }
    }
    const good = result.shots.filter((x) => x.good).length;
    if (good === SHOT_TIMES_S.length) { result.status = 'PASS'; result.reason = `world rendered (${att.name}); all screenshots good`; }
    else if (good) { result.status = 'FAIL'; result.reason = `only ${good}/${SHOT_TIMES_S.length} good screenshots`; }
    else { result.reason = 'world rendered but every screenshot was blank'; }
    break;
  }
  await browser.close().catch(() => {});
}

function finish() {
if (finished) return; finished = true;
writeLedger();
const d = dataSummary();
const md = [
  `# wenilla CI screenshot run: ${result.status}`,
  '',
  `- reason: ${result.reason}`,
  `- url: \`${URL_}\``,
  `- Chromium launch config that exposed WebGPU: ${result.flagSet ?? 'none'}`,
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
}

// Periodic ledger flush, and a hard stop so the script can never outlive 15 minutes.
setInterval(() => { try { writeLedger(); } catch {} }, 30000).unref();
const HARD_MS = Number(process.env.HARD_LIMIT_S || 900) * 1000;
setTimeout(() => { result.reason = `hard time limit ${HARD_MS / 1000}s reached: ${result.reason}`; note(result.reason); finish(); }, HARD_MS - 5000);

try { await main(); }
catch (e) { result.reason = `script error: ${e && e.stack || e}`; note(result.reason); }
finish();
