import { reloj, traducir } from '../core/i18n';
import type { Estado } from '../core/motor';
import type { Entrada, Idioma } from '../core/recetas';
import { Espectro } from './Espectro';
import { DE_SALA } from './estilos';
import { Alto, Pausa, Tocar } from './iconos';

interface Props {
  entrada: Entrada;
  estado: Estado;
  segundos: number;
  idioma: Idioma;
  alPausar: () => void;
  alReanudar: () => void;
  alParar: () => void;
  alExtender: () => void;
}

const COLOR = { comprobado: '#5ed6b0', prometedor: '#f0be64', exploracion: '#b096ff', propio: '#82beff' } as const;

/** La barra de lo que suena: cuánto queda, pausa, +15 min y parar. */
export function Reproductor({ entrada, estado, segundos, idioma, alPausar, alReanudar, alParar, alExtender }: Props) {
  const r = entrada.receta;
  const s = DE_SALA[r.sala];
  const t = (c: string) => traducir(idioma, c);
  const avance = estado.duracion > 0 ? Math.min(1, segundos / estado.duracion) : 0;
  const linea = estado.cargando
    ? t('cargando')
    : `${estado.sonando ? '' : `${t('enPausa')} · `}${t('quedan')} ${reloj(estado.duracion - segundos)}`;

  return (
    <div className="fixed inset-x-0 bottom-0 z-10 border-t border-borde bg-panel/95 pb-[env(safe-area-inset-bottom)] backdrop-blur">
      {!estado.simulado && (
        <div className={`mx-auto max-w-xl px-4 pt-2 ${s.texto}`}>
          <Espectro activo={estado.sonando && !estado.cargando} color={COLOR[r.sala]} etiqueta={t('espectro')} />
        </div>
      )}
      <div className="h-1 w-full bg-borde/60" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(avance * 100)}>
        <div className={`h-full ${s.barra}`} style={{ width: `${avance * 100}%` }} />
      </div>
      <div className="mx-auto flex max-w-xl items-center gap-2 px-4 py-3">
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium">{r.nombre[idioma]}</p>
          <p className="truncate text-sm tabular-nums text-tenue" aria-live="off">
            {linea}
          </p>
        </div>
        <button
          type="button"
          onClick={alExtender}
          className="min-h-11 rounded-full border border-borde px-3 text-sm font-medium text-texto/90"
        >
          {t('mas15')}
        </button>
        <button
          type="button"
          onClick={estado.sonando ? alPausar : alReanudar}
          aria-label={estado.sonando ? t('pausar') : t('reanudar')}
          className={`flex h-12 w-12 items-center justify-center rounded-full ${s.boton}`}
        >
          {estado.sonando ? <Pausa className="h-6 w-6" /> : <Tocar className="ml-0.5 h-6 w-6" />}
        </button>
        <button
          type="button"
          onClick={alParar}
          aria-label={t('parar')}
          className="flex h-11 w-11 items-center justify-center rounded-full border border-borde text-tenue"
        >
          <Alto className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}
