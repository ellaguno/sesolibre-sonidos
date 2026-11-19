import { minutosLegibles, traducir } from '../core/i18n';
import { necesitaAuriculares, type Entrada, type Idioma } from '../core/recetas';
import { DE_SALA } from './estilos';
import { Auriculares, Reloj, Tocar } from './iconos';

interface Props {
  entrada: Entrada;
  idioma: Idioma;
  sonando: boolean;
  alAbrir: () => void;
}

export function Tarjeta({ entrada, idioma, sonando, alAbrir }: Props) {
  const r = entrada.receta;
  const s = DE_SALA[r.sala];
  return (
    <button
      type="button"
      onClick={alAbrir}
      className={`group flex w-full items-center gap-4 rounded-2xl border bg-panel/80 p-4 text-left transition active:scale-[0.99] ${s.borde} ${sonando ? 'ring-1 ring-inset ring-white/20' : ''}`}
    >
      <span className={`flex h-12 w-12 shrink-0 items-center justify-center rounded-full ${s.fondo} ${s.texto}`}>
        {sonando ? <Ondas /> : <Tocar className="ml-0.5 h-6 w-6" />}
      </span>
      <span className="min-w-0 flex-1">
        <span className={`block text-xs font-semibold uppercase tracking-wider ${s.texto}`}>
          {traducir(idioma, `situacion.${r.situacion}`)}
        </span>
        <span className="mt-0.5 block text-lg font-medium leading-snug text-texto">{r.nombre[idioma]}</span>
        <span className="mt-1.5 flex items-center gap-3 text-sm text-tenue">
          <span className="inline-flex items-center gap-1">
            <Reloj className="h-4 w-4" />
            {minutosLegibles(idioma, Math.round(r.duracion_s / 60))}
          </span>
          {necesitaAuriculares(r) && (
            <span className="inline-flex items-center gap-1">
              <Auriculares className="h-4 w-4" />
              <span className="sr-only">{traducir(idioma, 'adv.auriculares')}</span>
            </span>
          )}
        </span>
      </span>
    </button>
  );
}

/** Tres barras que se mecen: esta receta es la que suena. */
function Ondas() {
  return (
    <span className="flex h-6 items-center gap-1" aria-hidden="true">
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className="w-1 animate-pulse rounded-full bg-current"
          style={{ height: `${[55, 100, 70][i]}%`, animationDelay: `${i * 240}ms`, animationDuration: '1.6s' }}
        />
      ))}
    </span>
  );
}
