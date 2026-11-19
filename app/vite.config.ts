import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [react()],
  // Las recetas viven en la raíz del repositorio (../recetas): son el contrato con el núcleo.
  server: { fs: { allow: ['..'] } },
  test: { environment: 'jsdom', include: ['tests/**/*.test.ts'] },
});
