import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import react from '@vitejs/plugin-react';
import { VitePWA } from 'vite-plugin-pwa';
import { defineConfig } from 'vitest/config';

// La versión es la de package.json; el pie de la app la muestra.
const pkg = JSON.parse(readFileSync(fileURLToPath(new URL('./package.json', import.meta.url)), 'utf-8')) as { version: string };

export default defineConfig({
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  // Rutas relativas: el mismo build sirve en la raíz de un dominio, en una subcarpeta de
  // GitHub Pages y dentro del WebView de Capacitor.
  base: './',
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.svg', 'icons/apple-touch-icon.png'],
      manifest: {
        name: 'SesoLibre Sonidos',
        short_name: 'Sonidos',
        description: 'Paisajes sonoros con propósito: dormir, concentrar, meditar, respirar.',
        lang: 'es',
        theme_color: '#0b1020',
        background_color: '#0b1020',
        display: 'standalone',
        orientation: 'portrait',
        icons: [
          { src: 'icons/icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: 'icons/icon-512.png', sizes: '512x512', type: 'image/png' },
          { src: 'icons/icon-512-maskable.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
        ],
      },
      workbox: {
        // La app y el núcleo (wasm) quedan en el teléfono al instalar. Las muestras (unos
        // 30 MB) no: cada una se guarda la primera vez que una receta la usa.
        globPatterns: ['**/*.{js,css,html,svg,png,wasm}', 'motor/worklet.js', 'audio/banco.json'],
        maximumFileSizeToCacheInBytes: 4 * 1024 * 1024,
        runtimeCaching: [
          {
            urlPattern: /\/audio\/.*\.ogg$/,
            handler: 'CacheFirst',
            options: { cacheName: 'sonidos-muestras', expiration: { maxEntries: 400 }, cacheableResponse: { statuses: [0, 200] } },
          },
        ],
      },
    }),
  ],
  // Las recetas viven en la raíz del repositorio (../recetas): son el contrato con el núcleo.
  server: { fs: { allow: ['..'] } },
  test: { environment: 'jsdom', include: ['tests/**/*.test.ts'] },
});
