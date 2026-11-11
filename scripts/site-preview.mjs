#!/usr/bin/env node
// The page's link preview (PLEXUS-62): site/og.png, 1200 × 630, is the page itself — in the dark, the pane part
// risen, the top of src/main.rs in it — photographed in headless Chromium. Run it again when the page changes:
//
//   node scripts/site-preview.mjs [--out site/og.png] [--rise 0..1] [--top VH] [--playwright DIR]
//
// Playwright is not a dependency of this repository: --playwright (or $PLAYWRIGHT_FROM) names a directory whose
// node_modules has it. The page is served from site/ on a loopback port of the system's choosing, for the shot alone.
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'site');
const arg = (name, or) => { const i = process.argv.indexOf(name); return i < 0 ? or : process.argv[i + 1]; };
const out = path.resolve(arg('--out', path.join(root, 'og.png')));
const rise = Number(arg('--rise', '0'));
const top = Number(arg('--top', '56'));
const from = arg('--playwright', process.env.PLAYWRIGHT_FROM || process.cwd());
const { chromium } = createRequire(path.join(path.resolve(from), 'x.js'))('playwright');

// The card is 1200 × 630. The page is laid out at half that size and photographed at two device pixels to one,
// so its phone layout fills the card and its source reads at a thumbnail's size.
const W = 600, H = 315, SCALE = 2;
const TYPES = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.txt': 'text/plain' };

const server = http.createServer((req, res) => {
  const rel = decodeURIComponent(new URL(req.url, 'http://x').pathname).replace(/^\/+/, '') || 'index.html';
  const file = path.join(root, rel);
  if (!file.startsWith(root + path.sep) || !fs.existsSync(file)) { res.writeHead(404).end(); return; }
  res.writeHead(200, { 'content-type': TYPES[path.extname(file)] || 'application/octet-stream' }).end(fs.readFileSync(file));
});
await new Promise(r => server.listen(0, '127.0.0.1', r));

const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
try {
  // reduced motion: the web holds still at its first moment, so the picture is the same every run
  const page = await browser.newPage({ viewport: { width: W, height: H }, deviceScaleFactor: SCALE, colorScheme: 'dark', reducedMotion: 'reduce' });
  const errors = [];
  page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
  page.on('pageerror', e => errors.push(String(e)));
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.evaluate(() => document.fonts.ready);
  if (await page.evaluate(() => document.documentElement.classList.contains('nogl'))) throw new Error('no WebGL: the web would not be drawn');
  // the caps (the mark, the links, the rate, `cargo add`) are the site's furniture, not the picture's
  // and the pane waits just under the web's dot, where a wide card has room for the whole of the first type
  await page.addStyleTag({ content: '.cap{display:none!important}.pane{top:' + top + 'vh!important}' });
  // one web, none beside it (main.js reads data-one); a resize has it drawn again
  await page.evaluate(() => { document.documentElement.setAttribute('data-one', ''); dispatchEvent(new Event('resize')); });
  // the scroll's first stretch raises the pane; --rise is how much of it
  await page.evaluate(r => { const t = document.getElementById('track'); t.scrollTop = Math.round(r * innerHeight * .52); }, rise);
  await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
  await page.evaluate(() => window.__draw && window.__draw());
  if (errors.length) throw new Error('the page said: ' + errors.join('; '));
  await page.screenshot({ path: out, type: 'png' });
  console.log(`${out}: ${W * SCALE} × ${H * SCALE}`);
} finally { await browser.close(); server.close(); }
