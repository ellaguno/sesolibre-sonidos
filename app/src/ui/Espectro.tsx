import { useEffect, useRef, useState } from 'react';

import { Sonidos } from '../core/motor';

/** Las bandas van de 40 Hz a 16 kHz en escala logarítmica (igual que en el núcleo). */
const F_MIN = 40;
const F_MAX = 16_000;
const MARCAS = [50, 100, 200, 500, 1000, 2000, 5000, 10_000];

export const hzDeBanda = (i: number, bandas: number) => F_MIN * (F_MAX / F_MIN) ** (i / bandas);
const xDeHz = (hz: number) => Math.log(hz / F_MIN) / Math.log(F_MAX / F_MIN);

export function hzLegible(hz: number): string {
  return hz >= 1000 ? `${(hz / 1000).toFixed(hz >= 10_000 ? 0 : 1).replace('.0', '')} kHz` : `${Math.round(hz)} Hz`;
}

/** Graves cálidos, medios verdes, agudos violetas: el color dice dónde está el sonido. */
const TINTES: [number, string][] = [
  [0, '240,150,90'],
  [0.3, '240,200,100'],
  [0.55, '94,214,176'],
  [0.8, '110,170,255'],
  [1, '176,150,255'],
];

interface Props {
  activo: boolean;
  etiqueta: string;
  textos: { pico: string; toca: string; simulado?: string };
}

/**
 * El espectrómetro. Las bandas las calcula el núcleo (FFT en Rust) sobre lo que de verdad
 * sale por el altavoz; aquí solo se dibujan, como un área de color con su línea de picos.
 * Pequeño es un adorno; al tocarlo se amplía y enseña el eje de frecuencias, la banda más
 * fuerte y, si se toca dentro, qué frecuencias cubre ese punto. Solo pregunta con la
 * pantalla a la vista: apagada no cuesta nada.
 */
export function Espectro({ activo, etiqueta, textos }: Props) {
  const lienzo = useRef<HTMLCanvasElement>(null);
  const [grande, setGrande] = useState(false);
  const [pico, setPico] = useState<number | null>(null);
  const [dedo, setDedo] = useState<number | null>(null);
  const bandas = grande ? 64 : 40;
  const dedoRef = useRef<number | null>(null);
  dedoRef.current = dedo;

  useEffect(() => {
    const c = lienzo.current;
    const ctx = c?.getContext('2d');
    if (!c || !ctx) return;
    const niveles = new Float32Array(bandas);
    const techos = new Float32Array(bandas);
    let vivo = true;
    let pidiendo = false;
    let cuenta = 0;

    const dibujar = () => {
      const dpr = window.devicePixelRatio || 1;
      const w = c.clientWidth * dpr;
      const h = c.clientHeight * dpr;
      if (c.width !== w || c.height !== h) {
        c.width = w;
        c.height = h;
      }
      ctx.clearRect(0, 0, w, h);
      const base = grande ? h - 18 * dpr : h;
      const x = (i: number) => ((i + 0.5) / bandas) * w;
      const y = (v: number) => base - v * (base - 4 * dpr);

      if (grande) {
        ctx.font = `${11 * dpr}px system-ui, sans-serif`;
        ctx.textAlign = 'center';
        for (const hz of MARCAS) {
          const px = xDeHz(hz) * w;
          ctx.strokeStyle = 'rgba(148,160,190,0.18)';
          ctx.lineWidth = dpr;
          ctx.beginPath();
          ctx.moveTo(px, 0);
          ctx.lineTo(px, base);
          ctx.stroke();
          ctx.fillStyle = 'rgba(148,160,190,0.9)';
          ctx.fillText(hz >= 1000 ? `${hz / 1000}k` : String(hz), px, h - 4 * dpr);
        }
      }

      const trazo = (valores: Float32Array) => {
        ctx.beginPath();
        ctx.moveTo(0, y(valores[0]));
        for (let i = 0; i < bandas - 1; i++) {
          const mx = (x(i) + x(i + 1)) / 2;
          ctx.quadraticCurveTo(x(i), y(valores[i]), mx, (y(valores[i]) + y(valores[i + 1])) / 2);
        }
        ctx.lineTo(w, y(valores[bandas - 1]));
      };
      const tinte = (alfa: number) => {
        const g = ctx.createLinearGradient(0, 0, w, 0);
        for (const [p, rgb] of TINTES) g.addColorStop(p, `rgba(${rgb},${alfa})`);
        return g;
      };

      // El área: color por frecuencia, más opaca arriba que abajo.
      trazo(niveles);
      ctx.lineTo(w, base);
      ctx.lineTo(0, base);
      ctx.closePath();
      ctx.fillStyle = tinte(0.55);
      ctx.fill();
      const velo = ctx.createLinearGradient(0, 0, 0, base);
      velo.addColorStop(0, 'rgba(20,27,48,0)');
      velo.addColorStop(1, 'rgba(20,27,48,0.75)');
      ctx.fillStyle = velo;
      ctx.fill();
      // El borde vivo y, más tenue, la línea de los picos recientes.
      trazo(niveles);
      ctx.strokeStyle = tinte(1);
      ctx.lineWidth = 2 * dpr;
      ctx.stroke();
      trazo(techos);
      ctx.strokeStyle = tinte(0.35);
      ctx.lineWidth = dpr;
      ctx.stroke();

      const d = dedoRef.current;
      if (grande && d !== null) {
        ctx.strokeStyle = 'rgba(226,231,245,0.8)';
        ctx.lineWidth = dpr;
        ctx.beginPath();
        ctx.moveTo(d * w, 0);
        ctx.lineTo(d * w, base);
        ctx.stroke();
      }
    };

    const tic = async () => {
      if (!vivo || pidiendo || document.visibilityState !== 'visible') return;
      pidiendo = true;
      try {
        const r = activo ? await Sonidos.espectro({ bandas }) : { bandas: [] as number[] };
        let mayor = 0;
        for (let i = 0; i < bandas; i++) {
          const nuevo = r.bandas[i] ?? 0;
          // Sube de golpe, baja despacio: se lee mejor.
          niveles[i] = nuevo > niveles[i] ? nuevo : niveles[i] * 0.8 + nuevo * 0.2;
          techos[i] = Math.max(niveles[i], techos[i] - 0.006);
          if (niveles[i] > niveles[mayor]) mayor = i;
        }
        if (!vivo) return;
        dibujar();
        if (grande && cuenta++ % 5 === 0) setPico(niveles[mayor] > 0.15 ? mayor : null);
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
  }, [activo, bandas, grande]);

  const rango = (i: number) => `${hzLegible(hzDeBanda(i, bandas))} – ${hzLegible(hzDeBanda(i + 1, bandas))}`;
  const alTocar = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!grande) return setGrande(true);
    const caja = e.currentTarget.getBoundingClientRect();
    const fx = Math.min(0.999, Math.max(0, (e.clientX - caja.left) / caja.width));
    setDedo((d) => (d !== null && Math.abs(d - fx) < 0.03 ? null : fx));
  };

  return (
    <div>
      <canvas
        ref={lienzo}
        role="img"
        aria-label={etiqueta}
        onPointerDown={alTocar}
        className={`block w-full cursor-pointer touch-none transition-[height] duration-200 ${grande ? 'h-52' : 'h-12'}`}
      />
      {grande && (
        <div className="flex items-center gap-3 pb-1 pt-1 text-xs tabular-nums text-tenue">
          <span className="min-w-0 flex-1 truncate">
            {dedo !== null
              ? rango(Math.floor(dedo * bandas))
              : pico !== null
                ? `${textos.pico}: ${rango(pico)}`
                : textos.toca}
            {textos.simulado ? ` · ${textos.simulado}` : ''}
          </span>
          <button type="button" onClick={() => (setGrande(false), setDedo(null))} className="min-h-9 rounded-full border border-borde px-3 font-medium">
            ▾
          </button>
        </div>
      )}
    </div>
  );
}
