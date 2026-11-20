import { useEffect, useRef } from 'react';

import { Sonidos } from '../core/motor';

const BANDAS = 48;

/**
 * El espectrómetro: las bandas las calcula el núcleo (FFT en Rust) sobre lo que de verdad
 * sale por el altavoz; aquí solo se dibujan. Solo pregunta mientras la pantalla está a la
 * vista: con la pantalla apagada no cuesta nada.
 */
export function Espectro({ activo, color, etiqueta }: { activo: boolean; color: string; etiqueta: string }) {
  const lienzo = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const c = lienzo.current;
    if (!c) return;
    const ctx = c.getContext('2d');
    if (!ctx) return;
    const niveles = new Float32Array(BANDAS);
    let vivo = true;
    let pidiendo = false;

    const dibujar = () => {
      const dpr = window.devicePixelRatio || 1;
      const w = c.clientWidth * dpr;
      const h = c.clientHeight * dpr;
      if (c.width !== w || c.height !== h) {
        c.width = w;
        c.height = h;
      }
      ctx.clearRect(0, 0, w, h);
      ctx.fillStyle = color;
      const paso = w / BANDAS;
      for (let i = 0; i < BANDAS; i++) {
        const alto = Math.max(dpr, niveles[i] * h);
        ctx.globalAlpha = 0.35 + 0.65 * niveles[i];
        ctx.fillRect(i * paso + paso * 0.15, h - alto, paso * 0.7, alto);
      }
    };

    const tic = async () => {
      if (!vivo || pidiendo || document.visibilityState !== 'visible') return;
      pidiendo = true;
      try {
        const { bandas } = activo ? await Sonidos.espectro({ bandas: BANDAS }) : { bandas: [] as number[] };
        for (let i = 0; i < BANDAS; i++) {
          const nuevo = bandas[i] ?? 0;
          // Sube de golpe, baja despacio: se lee mejor.
          niveles[i] = nuevo > niveles[i] ? nuevo : niveles[i] * 0.82 + nuevo * 0.18;
        }
        if (vivo) dibujar();
      } catch {
        /* el servicio aún no está: se reintenta en el siguiente tic */
      } finally {
        pidiendo = false;
      }
    };
    const id = setInterval(tic, 70);
    return () => {
      vivo = false;
      clearInterval(id);
    };
  }, [activo, color]);

  return <canvas ref={lienzo} role="img" aria-label={etiqueta} className="block h-14 w-full" />;
}
