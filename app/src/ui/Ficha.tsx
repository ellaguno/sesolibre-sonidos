import { useEffect, useState } from 'react';

import { minutosLegibles, traducir } from '../core/i18n';
import { duraciones, necesitaAuriculares, type Entrada, type Idioma } from '../core/recetas';
import { DE_SALA } from './estilos';
import { Cerrar, MarcaSala, Tocar } from './iconos';

interface Props {
  entrada: Entrada;
  idioma: Idioma;
  auriculares: boolean;
  /** En el navegador no se sabe si hay auriculares: no se avisa. */
  simulado: boolean;
  alCerrar: () => void;
  alTocar: (minutos: number) => void;
}

/** La ficha de una receta: qué es, qué dice la ciencia, advertencias, duración y tocar. */
export function Ficha({ entrada, idioma, auriculares, simulado, alCerrar, alTocar }: Props) {
  const r = entrada.receta;
  const s = DE_SALA[r.sala];
  const t = (c: string) => traducir(idioma, c);
  const [minutos, setMinutos] = useState(Math.round(r.duracion_s / 60));

  useEffect(() => {
    const alTeclear = (e: KeyboardEvent) => e.key === 'Escape' && alCerrar();
    window.addEventListener('keydown', alTeclear);
    return () => window.removeEventListener('keydown', alTeclear);
  }, [alCerrar]);

  return (
    <div className="fixed inset-0 z-20 flex items-end justify-center bg-black/60 sm:items-center" onClick={alCerrar}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label={r.nombre[idioma]}
        onClick={(e) => e.stopPropagation()}
        className="max-h-[92vh] w-full max-w-xl overflow-y-auto rounded-t-3xl border border-borde bg-panel p-5 pb-[max(1.25rem,env(safe-area-inset-bottom))] sm:rounded-3xl"
      >
        <div className="flex items-start gap-3">
          <div className="min-w-0 flex-1">
            <p className={`flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider ${s.texto}`}>
              <MarcaSala sala={r.sala} className="h-4 w-4" />
              {t(`sala.${r.sala}`)} · {t(`situacion.${r.situacion}`)}
            </p>
            <h2 className="mt-1 text-2xl font-semibold leading-tight">{r.nombre[idioma]}</h2>
          </div>
          <button
            type="button"
            onClick={alCerrar}
            aria-label={t('cerrar')}
            className="-mr-1 -mt-1 flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-tenue hover:text-texto"
          >
            <Cerrar className="h-6 w-6" />
          </button>
        </div>

        <p className="mt-3 leading-relaxed text-texto/90">{r.descripcion[idioma]}</p>

        <section className={`mt-4 rounded-2xl border p-4 ${s.borde} ${s.fondo}`}>
          <h3 className={`text-sm font-semibold ${s.texto}`}>{t('ciencia')}</h3>
          <p className="mt-1 text-sm leading-relaxed text-texto/90">{r.ciencia[idioma]}</p>
          {r.fuentes.length > 0 && (
            <p className="mt-2 text-sm text-tenue">
              {t('fuentes')}:{' '}
              {r.fuentes.map((f, i) => (
                <a key={f} href={f} target="_blank" rel="noreferrer" className="underline underline-offset-2">
                  {i > 0 && ' '}[{i + 1}]
                </a>
              ))}
            </p>
          )}
        </section>

        <ul className="mt-4 space-y-1 text-sm text-tenue">
          {r.advertencias.map((a) => (
            <li key={a}>· {t(`adv.${a}`)}</li>
          ))}
          <li>· {t('adv.siempre')}</li>
        </ul>

        {necesitaAuriculares(r) && !auriculares && !simulado && (
          <p className="mt-3 rounded-xl border border-prometedor/40 bg-prometedor/10 p-3 text-sm text-prometedor">
            {t('sinAuriculares')}
          </p>
        )}

        <h3 className="mt-5 text-sm font-semibold text-tenue">{t('duracion')}</h3>
        <div className="mt-2 flex flex-wrap gap-2">
          {duraciones(r).map((m) => (
            <button
              key={m}
              type="button"
              onClick={() => setMinutos(m)}
              aria-pressed={m === minutos}
              className={`min-h-11 rounded-full border px-4 text-sm font-medium transition ${
                m === minutos ? `${s.boton} border-transparent` : 'border-borde text-texto/90'
              }`}
            >
              {minutosLegibles(idioma, m)}
            </button>
          ))}
        </div>

        <button
          type="button"
          onClick={() => alTocar(minutos)}
          className={`mt-6 flex min-h-14 w-full items-center justify-center gap-2 rounded-full text-lg font-semibold ${s.boton}`}
        >
          <Tocar className="h-6 w-6" />
          {t('tocar')} · {minutosLegibles(idioma, minutos)}
        </button>
      </div>
    </div>
  );
}
