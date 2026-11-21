// Capturas para el App Store: la misma web que corre dentro de la app de iOS, en Chrome sin
// pantalla, al tamaño que pide Apple (iPhone 6,9" y iPad 13"), en español e inglés.
// Sin dependencias: habla con Chrome por el protocolo DevTools (WebSocket de Node 22).
//
//   cd app && npm run build && npx vite preview --port 4173 &
//   node tools/capturas-ios.mjs            # → assets/appstore/<idioma>/<equipo>-<n>.png
import { spawn } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const RAIZ = join(dirname(fileURLToPath(import.meta.url)), '..');
const URL_APP = process.env.URL_APP ?? 'http://localhost:4173/';
const CHROME = process.env.CHROME ?? 'google-chrome';
const EQUIPOS = {
  iphone: { width: 440, height: 956, deviceScaleFactor: 3, mobile: true }, // 1320×2868
  ipad: { width: 1024, height: 1366, deviceScaleFactor: 2, mobile: true }, // 2048×2732
};
// Cada escena: lo que se hace en la página antes de la captura (texto visible a pulsar).
const ESCENAS = {
  es: [
    ['inicio', []],
    ['receta', ['Claro del bosque']],
    ['sonando', ['Claro del bosque', 'Tocar']],
    ['disenar', ['Diseñar']],
  ],
  en: [
    ['inicio', []],
    ['receta', ['Forest clearing']],
    ['sonando', ['Forest clearing', 'Play']],
    ['disenar', ['Design']],
  ],
};

const espera = (ms) => new Promise((r) => setTimeout(r, ms));
const perfil = mkdtempSync(join(tmpdir(), 'capturas-'));
const chrome = spawn(CHROME, [
  '--headless=new', '--remote-debugging-port=9333', `--user-data-dir=${perfil}`,
  '--autoplay-policy=no-user-gesture-required', '--hide-scrollbars', 'about:blank',
], { stdio: 'ignore' });

let objetivos;
for (let i = 0; i < 50 && !objetivos; i++) {
  await espera(200);
  objetivos = await fetch('http://127.0.0.1:9333/json').then((r) => r.json()).catch(() => null);
}
const ws = new WebSocket(objetivos.find((t) => t.type === 'page').webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r));
let n = 0;
const pendientes = new Map();
ws.addEventListener('message', (e) => {
  const m = JSON.parse(e.data);
  if (m.id && pendientes.has(m.id)) pendientes.get(m.id)(m), pendientes.delete(m.id);
});
const cdp = (method, params = {}) =>
  new Promise((r) => { const id = ++n; pendientes.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
const js = async (expr) => (await cdp('Runtime.evaluate', { expression: expr, awaitPromise: true })).result?.result?.value;
// Pulsa el botón (o tarjeta) más interno cuyo texto empieza por `texto`.
const pulsar = (texto) => js(`(() => {
  const els = [...document.querySelectorAll('button, [role=button], a')].filter(e => e.offsetParent && e.innerText.trim().includes(${JSON.stringify(texto)}));
  els.sort((a, b) => a.innerText.length - b.innerText.length);
  if (!els[0]) return false; els[0].scrollIntoView({block: 'center'}); els[0].click(); return true; })()`);

for (const [idioma, escenas] of Object.entries(ESCENAS)) {
  for (const [equipo, metrica] of Object.entries(EQUIPOS)) {
    const carpeta = join(RAIZ, 'assets/appstore', idioma);
    mkdirSync(carpeta, { recursive: true });
    await cdp('Emulation.setDeviceMetricsOverride', metrica);
    await cdp('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: 'dark' }] });
    for (const [i, [nombre, pasos]] of escenas.entries()) {
      await cdp('Page.navigate', { url: URL_APP });
      await espera(2500);
      // El idioma se elige con el botón ES/EN, que la app recuerda.
      const actual = await js(`document.documentElement.lang || 'es'`);
      if (!actual.startsWith(idioma)) { await pulsar(idioma === 'en' ? 'EN' : 'ES'); await espera(600); }
      await js('window.scrollTo(0, 0)');
      for (const paso of pasos) {
        if (!(await pulsar(paso))) console.warn(`  ${idioma}/${equipo}/${nombre}: no encontré «${paso}»`);
        await espera(1800);
      }
      if (!pasos.length || nombre === 'sonando') await js('window.scrollTo(0, 0)');
      await espera(800);
      const { result } = await cdp('Page.captureScreenshot', { format: 'png' });
      const archivo = join(carpeta, `${equipo}-${i + 1}-${nombre}.png`);
      writeFileSync(archivo, Buffer.from(result.data, 'base64'));
      console.log(archivo.replace(RAIZ + '/', ''));
    }
  }
}
ws.close();
chrome.kill();
