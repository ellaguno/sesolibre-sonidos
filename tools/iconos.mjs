// Rasteriza los SVG de assets/icono con Chromium (Playwright): node tools/iconos.mjs
import { readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const { chromium } = createRequire(import.meta.url)('playwright');
const trabajos = process.argv.slice(2).map((a) => a.split(':')); // svg:png:tamaño
const b = await chromium.launch();
const p = await b.newPage();
for (const [svg, png, lado] of trabajos) {
  const n = Number(lado);
  await p.setViewportSize({ width: n, height: n });
  await p.setContent(`<body style="margin:0;background:transparent">${readFileSync(join(raiz, svg), 'utf8').replace('width="1024" height="1024"', `width="${n}" height="${n}"`)}</body>`);
  writeFileSync(join(raiz, png), await p.screenshot({ omitBackground: true }));
  console.log(png, n);
}
await b.close();
