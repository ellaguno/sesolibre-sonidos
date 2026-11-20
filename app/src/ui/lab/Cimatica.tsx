import { useCallback, useEffect, useRef, useState } from 'react';

import { borrar, guardar, idNuevo, leerTodo, type FotoCimatica } from '../../core/almacen';
import { aJpeg, exportar } from '../../core/foto';
import { traducir } from '../../core/i18n';
import { Sonidos, type Estado } from '../../core/motor';
import { RECETAS, type Idioma } from '../../core/recetas';
import { Espectro } from '../Espectro';

const RECETA = RECETAS.find((e) => e.receta.id === 'laboratorio.cimatica')!;
const MIN = 20;
const MAX = 2000;
/** El deslizador va en escala logarítmica: los graves, que es donde responde el agua, ocupan más. */
const aHz = (x: number) => MIN * (MAX / MIN) ** x;
const aX = (hz: number) => Math.log(hz / MIN) / Math.log(MAX / MIN);
const PRESETS = [24, 32, 48, 64, 96, 128, 256, 432, 528];

interface Props {
  idioma: Idioma;
  estado: Estado;
}

/**
 * Generador de tonos con cámara. El tono lo hace el núcleo (capa `mando`), así que pasa por
 * el mismo limitador que todo lo demás; la fuerza empieza baja y la sube el usuario.
 */
export function Cimatica({ idioma, estado }: Props) {
  const t = (c: string) => traducir(idioma, c);
  const [hz, setHz] = useState(64);
  const [fuerza, setFuerza] = useState(-24);
  const [barrido, setBarrido] = useState(false);
  const [camara, setCamara] = useState<'cerrada' | 'abierta' | 'fallo'>('cerrada');
  const [fotos, setFotos] = useState<FotoCimatica[]>([]);
  const video = useRef<HTMLVideoElement>(null);
  const flujo = useRef<MediaStream | null>(null);
  const sonando = estado.activo && estado.recetaId === RECETA.receta.id;

  const recargar = useCallback(() => void leerTodo<FotoCimatica>('cimatica').then((f) => setFotos(f.sort((a, b) => b.fecha - a.fecha))), []);
  useEffect(recargar, [recargar]);

  const afinar = (nuevo: number) => {
    const acotado = Math.min(MAX, Math.max(MIN, Math.round(nuevo * 10) / 10));
    setHz(acotado);
    if (sonando) void Sonidos.mando({ hz: acotado });
  };

  // Barrido: un semitono cada dos segundos, hacia arriba, hasta el tope.
  useEffect(() => {
    if (!barrido || !sonando) return;
    const id = setInterval(() => {
      setHz((actual) => {
        const siguiente = Math.min(MAX, actual * 2 ** (1 / 96));
        void Sonidos.mando({ hz: siguiente });
        if (siguiente >= MAX) setBarrido(false);
        return Math.round(siguiente * 10) / 10;
      });
    }, 250);
    return () => clearInterval(id);
  }, [barrido, sonando]);

  const alternar = async () => {
    if (sonando) {
      setBarrido(false);
      await Sonidos.parar();
      return;
    }
    if (!window.confirm(t('cim.auriculares'))) return;
    await Sonidos.tocar({
      receta: RECETA.json,
      recetaId: RECETA.receta.id,
      titulo: RECETA.receta.nombre[idioma],
      subtitulo: 'SesoLibre Sonidos',
      minutos: 30,
      volumenDb: fuerza,
    });
    await Sonidos.mando({ hz });
  };

  const cerrarCamara = useCallback(() => {
    flujo.current?.getTracks().forEach((p) => p.stop());
    flujo.current = null;
    setCamara('cerrada');
  }, []);
  useEffect(() => cerrarCamara, [cerrarCamara]);

  const abrirCamara = async () => {
    try {
      const s = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' }, audio: false });
      flujo.current = s;
      setCamara('abierta');
      requestAnimationFrame(() => {
        if (video.current) {
          video.current.srcObject = s;
          void video.current.play();
        }
      });
    } catch {
      setCamara('fallo');
    }
  };

  const fotografiar = async () => {
    const v = video.current;
    if (!v || !v.videoWidth) return;
    const imagen = aJpeg(v, v.videoWidth, v.videoHeight, 1280, `${hz} Hz · SesoLibre Sonidos`);
    await guardar('cimatica', { id: idNuevo(), fecha: Date.now(), hz, imagen });
    recargar();
  };

  const boton = 'min-h-11 rounded-full border border-borde px-4 text-sm font-medium';
  return (
    <div className="space-y-5">
      <p className="text-tenue">{t('cim.que')}</p>
      <details className="rounded-2xl border border-borde bg-panel/70 p-4 text-sm">
        <summary className="cursor-pointer font-semibold">{t('cim.como')}</summary>
        <ol className="mt-2 list-decimal space-y-1.5 pl-5 text-texto/90">
          {['cim.p1', 'cim.p2', 'cim.p3', 'cim.p4'].map((k) => (
            <li key={k}>{t(k)}</li>
          ))}
        </ol>
      </details>

      <section className="rounded-2xl border border-dashed border-exploracion/40 bg-panel/80 p-4">
        <p className="text-center text-5xl font-semibold tabular-nums text-exploracion" aria-live="off">
          {hz.toFixed(hz < 100 ? 1 : 0)} <span className="text-2xl">Hz</span>
        </p>
        <label className="mt-3 block text-sm text-tenue">
          {t('cim.frecuencia')}
          <input
            type="range"
            min={0}
            max={1}
            step={0.001}
            value={aX(hz)}
            onChange={(e) => afinar(aHz(Number(e.target.value)))}
            className="mt-1 block w-full accent-[rgb(176,150,255)]"
          />
        </label>
        <div className="mt-2 flex flex-wrap justify-center gap-2">
          {[-10, -1, 1, 10].map((d) => (
            <button key={d} type="button" onClick={() => afinar(hz + d)} className={boton}>
              {d > 0 ? '+' : '−'}
              {Math.abs(d)}
            </button>
          ))}
          <button type="button" aria-pressed={barrido} disabled={!sonando} onClick={() => setBarrido((b) => !b)} className={`${boton} disabled:opacity-40 ${barrido ? 'border-transparent bg-exploracion text-fondo' : ''}`}>
            {t('cim.barrido')}
          </button>
        </div>
        <div className="mt-2 flex flex-wrap justify-center gap-2">
          {PRESETS.map((p) => (
            <button key={p} type="button" onClick={() => afinar(p)} className="min-h-9 rounded-full border border-borde px-3 text-xs tabular-nums text-tenue">
              {p}
            </button>
          ))}
        </div>
        <label className="mt-4 block text-sm text-tenue">
          {t('cim.fuerza')}
          <input
            type="range"
            min={-36}
            max={0}
            step={1}
            value={fuerza}
            onChange={(e) => {
              const db = Number(e.target.value);
              setFuerza(db);
              if (sonando) void Sonidos.volumen({ db });
            }}
            className="mt-1 block w-full accent-[rgb(176,150,255)]"
          />
        </label>
        <button type="button" onClick={() => void alternar()} className="mt-4 min-h-14 w-full rounded-full bg-exploracion text-lg font-semibold text-fondo">
          {sonando ? t('cim.callar') : t('cim.sonar')}
        </button>
        <p className="mt-2 text-center text-xs text-tenue">{t('adv.altavoz')}</p>
        {sonando && (
          <div className="mt-3">
            <Espectro activo etiqueta={t('espectro')} textos={{ pico: t('espectroPico'), toca: t('espectroToca') }} />
          </div>
        )}
      </section>

      <section>
        {camara === 'abierta' ? (
          <div className="space-y-2">
            <video ref={video} playsInline muted className="w-full rounded-2xl border border-borde bg-black" />
            <div className="flex gap-2">
              <button type="button" onClick={() => void fotografiar()} className="min-h-12 flex-1 rounded-full bg-exploracion font-semibold text-fondo">
                {t('cim.foto')} · {hz.toFixed(0)} Hz
              </button>
              <button type="button" onClick={cerrarCamara} className={boton}>
                {t('cim.cerrarCamara')}
              </button>
            </div>
          </div>
        ) : (
          <button type="button" onClick={() => void abrirCamara()} className={`${boton} w-full`}>
            {t('cim.camara')}
          </button>
        )}
        {camara === 'fallo' && <p className="mt-2 text-sm text-prometedor">{t('cim.sinCamara')}</p>}
      </section>

      {fotos.length > 0 && (
        <section>
          <h3 className="text-sm font-semibold uppercase tracking-wider text-tenue">{t('cim.fotos')}</h3>
          <ul className="mt-2 grid grid-cols-2 gap-3">
            {fotos.map((f) => (
              <li key={f.id} className="overflow-hidden rounded-xl border border-borde bg-panel">
                <img src={f.imagen} alt={`${f.hz} Hz`} className="aspect-video w-full bg-black object-contain" />
                <div className="flex items-center gap-1 p-2 text-xs">
                  <button type="button" onClick={() => afinar(f.hz)} className="flex-1 text-left font-semibold tabular-nums text-exploracion">
                    {f.hz} Hz
                  </button>
                  <button type="button" onClick={() => void exportar(`cimatica-${f.hz}Hz.jpg`, f.imagen, 'jpeg')} className="min-h-9 px-2 text-tenue underline">
                    {t('compartir')}
                  </button>
                  <button type="button" onClick={() => void borrar('cimatica', f.id).then(recargar)} className="min-h-9 px-2 text-red-200 underline">
                    {t('borrar')}
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
