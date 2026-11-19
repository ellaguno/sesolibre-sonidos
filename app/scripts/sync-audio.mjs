// Copia las muestras del repositorio (../assets/audio) a public/audio, de donde Vite las
// lleva a dist/ y Capacitor al APK (assets/public/audio). public/audio no se versiona.
import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const origen = join(raiz, '..', 'assets', 'audio');
const destino = join(raiz, 'public', 'audio');

rmSync(destino, { recursive: true, force: true });
mkdirSync(destino, { recursive: true });
for (const parte of ['instruments', 'ambience', 'banco.json']) {
  if (!existsSync(join(origen, parte))) throw new Error(`falta ${join(origen, parte)}`);
  cpSync(join(origen, parte), join(destino, parte), { recursive: true });
}
console.log(`audio: ${origen} → ${destino}`);
