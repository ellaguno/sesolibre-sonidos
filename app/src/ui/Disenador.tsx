import { useState } from 'react';

import { COLCHONES, DENSIDADES, EVENTOS, GRUPOS_DE_FONDO, FRECUENCIAS, INSTRUMENTOS, MODOS, type Diseno } from '../core/diseno';
import { traducir } from '../core/i18n';
import type { Idioma } from '../core/recetas';
import { DE_SALA } from './estilos';
import { Cerrar } from './iconos';

interface Props {
  inicial: Diseno;
  idioma: Idioma;
  alCerrar: () => void;
  alGuardar: (d: Diseno) => void;
}

/** El diseñador: fondo, colchón, instrumento, frecuencia y respiración, por fichas. */
export function Disenador({ inicial, idioma, alCerrar, alGuardar }: Props) {
  const [d, setD] = useState<Diseno>(inicial);
  const t = (c: string) => traducir(idioma, c);
  const s = DE_SALA.propio;
  const poner = (cambio: Partial<Diseno>) => setD((v) => ({ ...v, ...cambio }));
  const vacio =
    d.fondos.length === 0 && d.colchon === 'ninguno' && d.instrumento === 'ninguno' && d.frecuencia === 'ninguno' && !d.respiracion && (d.evento ?? 'ninguno') === 'ninguno';
  const conPulso = d.frecuencia === 'binaural' || d.frecuencia === 'isocronico';

  const ficha = (activa: boolean) =>
    `min-h-11 rounded-full border px-4 text-sm font-medium transition ${activa ? `${s.boton} border-transparent` : 'border-borde text-texto/90'}`;

  function Grupo<T extends string | number>({ titulo, opciones, valor, alElegir }: { titulo: string; opciones: readonly T[]; valor: T; alElegir: (v: T) => void }) {
    return (
      <fieldset className="mt-5">
        <legend className="text-sm font-semibold text-tenue">{t(titulo)}</legend>
        <div className="mt-2 flex flex-wrap gap-2">
          {opciones.map((o) => (
            <button key={o} type="button" aria-pressed={o === valor} onClick={() => alElegir(o)} className={ficha(o === valor)}>
              {t(`o.${o}`)}
            </button>
          ))}
        </div>
      </fieldset>
    );
  }

  const numero = (titulo: string, valor: number, min: number, max: number, paso: number, alCambiar: (n: number) => void) => (
    <label className="mt-3 block text-sm text-tenue">
      {t(titulo)}
      <input
        type="number"
        inputMode="decimal"
        min={min}
        max={max}
        step={paso}
        value={valor}
        onChange={(e) => alCambiar(Number(e.target.value))}
        className="mt-1 block min-h-11 w-40 rounded-xl border border-borde bg-fondo px-3 text-base text-texto"
      />
    </label>
  );

  return (
    <div className="fixed inset-0 z-20 flex items-end justify-center bg-black/60 sm:items-center" onClick={alCerrar}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t('disenar')}
        onClick={(e) => e.stopPropagation()}
        className="max-h-[94vh] w-full max-w-xl overflow-y-auto rounded-t-3xl border border-borde bg-panel p-5 pb-[max(1.25rem,env(safe-area-inset-bottom))] sm:rounded-3xl"
      >
        <div className="flex items-center gap-3">
          <h2 className={`flex-1 text-2xl font-semibold ${s.texto}`}>{t('disenar')}</h2>
          <button type="button" onClick={alCerrar} aria-label={t('cerrar')} className="flex h-11 w-11 items-center justify-center rounded-full text-tenue">
            <Cerrar className="h-6 w-6" />
          </button>
        </div>

        <label className="mt-3 block text-sm font-semibold text-tenue">
          {t('nombre')}
          <input
            type="text"
            maxLength={60}
            value={d.nombre}
            placeholder={t('miDiseno')}
            onChange={(e) => poner({ nombre: e.target.value })}
            className="mt-1 block min-h-11 w-full rounded-xl border border-borde bg-fondo px-3 text-base font-normal text-texto"
          />
        </label>

        <fieldset className="mt-5">
          <legend className="text-sm font-semibold text-tenue">{t('d.fondo')}</legend>
          {GRUPOS_DE_FONDO.map((g) => (
            <div key={g.titulo} className="mt-3">
              <p className="text-xs uppercase tracking-wider text-tenue/80">{t(g.titulo)}</p>
              <div className="mt-1.5 flex flex-wrap gap-2">
                {g.fondos.map((f) => {
                  const puesto = d.fondos.includes(f);
                  return (
                    <button
                      key={f}
                      type="button"
                      aria-pressed={puesto}
                      disabled={!puesto && d.fondos.length >= 3}
                      onClick={() => poner({ fondos: puesto ? d.fondos.filter((x) => x !== f) : [...d.fondos, f] })}
                      className={`${ficha(puesto)} disabled:opacity-40`}
                    >
                      {t(`o.${f}`)}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
        </fieldset>

        <Grupo titulo="d.colchon" opciones={COLCHONES} valor={d.colchon} alElegir={(colchon) => poner({ colchon })} />
        <Grupo titulo="d.instrumento" opciones={INSTRUMENTOS} valor={d.instrumento} alElegir={(instrumento) => poner({ instrumento })} />
        {d.instrumento !== 'ninguno' && (
          <Grupo
            titulo="d.densidad"
            opciones={Object.keys(DENSIDADES) as (keyof typeof DENSIDADES)[]}
            valor={d.densidad}
            alElegir={(densidad) => poner({ densidad })}
          />
        )}
        {(d.colchon !== 'ninguno' || d.instrumento !== 'ninguno') && (
          <>
            <Grupo titulo="d.modo" opciones={MODOS} valor={d.modo} alElegir={(modo) => poner({ modo })} />
            <Grupo titulo="d.afinacion" opciones={[440, 432] as const} valor={d.afinacion} alElegir={(afinacion) => poner({ afinacion })} />
          </>
        )}

        <Grupo titulo="d.frecuencia" opciones={FRECUENCIAS} valor={d.frecuencia} alElegir={(frecuencia) => poner({ frecuencia })} />
        {(d.frecuencia === 'tono' || conPulso) && numero('d.hz', d.hz, 40, d.frecuencia === 'tono' ? 4000 : 1000, 1, (hz) => poner({ hz }))}
        {(conPulso || d.frecuencia === 'serie') && numero('d.beat', d.beat, 0.5, d.frecuencia === 'serie' ? 200 : 45, 0.01, (beat) => poner({ beat }))}
        {d.frecuencia === 'binaural' && <p className="mt-2 text-sm text-prometedor">{t('adv.auriculares')}</p>}

        <Grupo titulo="d.evento" opciones={EVENTOS} valor={d.evento ?? 'ninguno'} alElegir={(evento) => poner({ evento })} />
        <Grupo titulo="d.respiracion" opciones={['no', 'si'] as const} valor={d.respiracion ? 'si' : 'no'} alElegir={(v) => poner({ respiracion: v === 'si' })} />

        {vacio && <p className="mt-4 text-sm text-prometedor">{t('d.vacio')}</p>}
        <button
          type="button"
          disabled={vacio}
          onClick={() => alGuardar(d)}
          className={`mt-6 flex min-h-14 w-full items-center justify-center rounded-full text-lg font-semibold disabled:opacity-40 ${s.boton}`}
        >
          {t('guardar')}
        </button>
      </div>
    </div>
  );
}
