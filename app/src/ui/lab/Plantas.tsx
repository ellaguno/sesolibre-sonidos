import { useCallback, useEffect, useState } from 'react';

import { aCsv, borrar, guardar, idNuevo, leerTodo, resumen, type Experimento, type Medicion } from '../../core/almacen';
import { archivoAJpeg, exportar } from '../../core/foto';
import { traducir } from '../../core/i18n';
import { Sonidos } from '../../core/motor';
import type { Entrada, Idioma } from '../../core/recetas';

interface Props {
  idioma: Idioma;
  /** Catálogo y diseños entre los que elegir el sonido diario. */
  recetas: Entrada[];
}

const campo = 'mt-1 block min-h-11 w-full rounded-xl border border-borde bg-fondo px-3 text-base text-texto';
const boton = 'min-h-11 rounded-full border border-borde px-4 text-sm font-medium';

/** Experimento con control: protocolo, sesión diaria, diario de medidas y fotos, y gráfica. */
export function Plantas({ idioma, recetas }: Props) {
  const t = (c: string) => traducir(idioma, c);
  const [lista, setLista] = useState<Experimento[]>([]);
  const [nombre, setNombre] = useState('');
  const [recetaId, setRecetaId] = useState(recetas[0]?.receta.id ?? '');
  const [minutos, setMinutos] = useState(60);

  const recargar = useCallback(() => void leerTodo<Experimento>('experimentos').then((e) => setLista(e.sort((a, b) => b.inicio - a.inicio))), []);
  useEffect(recargar, [recargar]);

  const crear = async () => {
    await guardar('experimentos', { id: idNuevo(), nombre: nombre.trim() || t('pl.titulo'), inicio: Date.now(), recetaId, minutosDia: minutos, mediciones: [] });
    setNombre('');
    recargar();
  };

  return (
    <div className="space-y-5">
      <p className="text-tenue">{t('pl.que')}</p>
      <details className="rounded-2xl border border-borde bg-panel/70 p-4 text-sm" open={lista.length === 0}>
        <summary className="cursor-pointer font-semibold">{t('pl.protocolo')}</summary>
        <ol className="mt-2 list-decimal space-y-1.5 pl-5 text-texto/90">
          {['pl.r1', 'pl.r2', 'pl.r3', 'pl.r4', 'pl.r5'].map((k) => (
            <li key={k}>{t(k)}</li>
          ))}
        </ol>
        <p className="mt-3 text-tenue">{t('pl.ciencia')}</p>
      </details>

      {lista.map((e) => (
        <Ficha key={e.id} e={e} idioma={idioma} entrada={recetas.find((r) => r.receta.id === e.recetaId)} alCambiar={recargar} />
      ))}

      <section className="rounded-2xl border border-dashed border-exploracion/40 bg-panel/80 p-4">
        <h3 className="font-semibold text-exploracion">{t('pl.nuevo')}</h3>
        <label className="mt-3 block text-sm text-tenue">
          {t('pl.nombre')}
          <input type="text" maxLength={60} value={nombre} onChange={(ev) => setNombre(ev.target.value)} className={campo} />
        </label>
        <label className="mt-3 block text-sm text-tenue">
          {t('pl.receta')}
          <select value={recetaId} onChange={(ev) => setRecetaId(ev.target.value)} className={campo}>
            {recetas.map((r) => (
              <option key={r.receta.id} value={r.receta.id}>
                {r.receta.nombre[idioma]}
              </option>
            ))}
          </select>
        </label>
        <label className="mt-3 block text-sm text-tenue">
          {t('pl.minutos')}
          <input type="number" min={5} max={240} value={minutos} onChange={(ev) => setMinutos(Math.min(240, Math.max(5, Number(ev.target.value) || 60)))} className={`${campo} w-32`} />
        </label>
        <button type="button" onClick={() => void crear()} className="mt-4 min-h-12 w-full rounded-full bg-exploracion font-semibold text-fondo">
          {t('pl.crear')}
        </button>
      </section>
    </div>
  );
}

function Ficha({ e, idioma, entrada, alCambiar }: { e: Experimento; idioma: Idioma; entrada?: Entrada; alCambiar: () => void }) {
  const t = (c: string) => traducir(idioma, c);
  const [control, setControl] = useState('');
  const [sonido, setSonido] = useState('');
  const [nota, setNota] = useState('');
  const [fotoControl, setFotoControl] = useState<string | undefined>();
  const [fotoSonido, setFotoSonido] = useState<string | undefined>();
  const dia = (fecha: number) => Math.round((fecha - e.inicio) / 86_400_000);
  const num = (s: string) => (s.trim() === '' || !Number.isFinite(Number(s.replace(',', '.'))) ? null : Number(s.replace(',', '.')));
  const r = resumen(e);

  const apuntar = async () => {
    const m: Medicion = { fecha: Date.now(), control: num(control), sonido: num(sonido), nota: nota.trim(), fotoControl, fotoSonido };
    if (m.control === null && m.sonido === null && !m.nota && !fotoControl && !fotoSonido) return;
    await guardar('experimentos', { ...e, mediciones: [...e.mediciones, m] });
    setControl('');
    setSonido('');
    setNota('');
    setFotoControl(undefined);
    setFotoSonido(undefined);
    alCambiar();
  };

  const sesion = () => {
    if (!entrada) return;
    void Sonidos.tocar({
      receta: entrada.json,
      recetaId: entrada.receta.id,
      titulo: entrada.receta.nombre[idioma],
      subtitulo: e.nombre,
      minutos: e.minutosDia,
    });
  };

  const foto = (poner: (s: string | undefined) => void) => (ev: React.ChangeEvent<HTMLInputElement>) => {
    const f = ev.target.files?.[0];
    if (f) void archivoAJpeg(f).then(poner);
  };

  return (
    <section className="rounded-2xl border border-borde bg-panel/80 p-4">
      <div className="flex items-baseline gap-2">
        <h3 className="flex-1 text-lg font-semibold">{e.nombre}</h3>
        <span className="text-sm tabular-nums text-tenue">
          {t('pl.dia')} {dia(Date.now())}
        </span>
      </div>
      <p className="text-sm text-tenue">
        {entrada?.receta.nombre[idioma] ?? e.recetaId} · {e.minutosDia} min
      </p>
      <button type="button" onClick={sesion} disabled={!entrada} className="mt-3 min-h-12 w-full rounded-full bg-exploracion font-semibold text-fondo disabled:opacity-40">
        {t('pl.sesion')} · {e.minutosDia} min
      </button>

      <Grafica e={e} etiquetas={[t('pl.control'), t('pl.sonido')]} vacio={t('pl.sinDatos')} />
      {r && (
        <p className="mt-2 text-sm">
          {t('pl.crecio').replace('{d}', String(r.dias)).replace('{c}', r.control.toFixed(1)).replace('{s}', r.sonido.toFixed(1))}{' '}
          <span className="text-tenue">{t('pl.ojo')}</span>
        </p>
      )}

      <div className="mt-4 grid grid-cols-2 gap-3">
        <label className="text-sm text-tenue">
          {t('pl.control')}
          <input type="text" inputMode="decimal" value={control} onChange={(ev) => setControl(ev.target.value)} className={campo} />
        </label>
        <label className="text-sm text-tenue">
          {t('pl.sonido')}
          <input type="text" inputMode="decimal" value={sonido} onChange={(ev) => setSonido(ev.target.value)} className={campo} />
        </label>
        <label className="text-sm text-tenue">
          {t('pl.fotoControl')} {fotoControl ? '✓' : ''}
          <input type="file" accept="image/*" capture="environment" onChange={foto(setFotoControl)} className="mt-1 block w-full text-xs" />
        </label>
        <label className="text-sm text-tenue">
          {t('pl.fotoSonido')} {fotoSonido ? '✓' : ''}
          <input type="file" accept="image/*" capture="environment" onChange={foto(setFotoSonido)} className="mt-1 block w-full text-xs" />
        </label>
      </div>
      <label className="mt-3 block text-sm text-tenue">
        {t('pl.nota')}
        <input type="text" maxLength={200} value={nota} onChange={(ev) => setNota(ev.target.value)} className={campo} />
      </label>
      <button type="button" onClick={() => void apuntar()} className={`${boton} mt-3 w-full`}>
        {t('pl.medir')}
      </button>

      {e.mediciones.length > 0 && (
        <ul className="mt-4 space-y-2 text-sm">
          {e.mediciones
            .slice()
            .sort((a, b) => b.fecha - a.fecha)
            .map((m) => (
              <li key={m.fecha} className="rounded-xl border border-borde p-2">
                <p className="tabular-nums">
                  <span className="text-tenue">
                    {t('pl.dia')} {dia(m.fecha)}
                  </span>{' '}
                  · {t('pl.control')}: {m.control ?? '—'} · {t('pl.sonido')}: {m.sonido ?? '—'}
                </p>
                {m.nota && <p className="text-tenue">{m.nota}</p>}
                {(m.fotoControl || m.fotoSonido) && (
                  <div className="mt-1 grid grid-cols-2 gap-2">
                    {[m.fotoControl, m.fotoSonido].map((f, i) =>
                      f ? <img key={i} src={f} alt={i === 0 ? t('pl.fotoControl') : t('pl.fotoSonido')} className="aspect-square w-full rounded-lg object-cover" /> : <span key={i} />,
                    )}
                  </div>
                )}
              </li>
            ))}
        </ul>
      )}

      <div className="mt-3 flex gap-2">
        <button type="button" onClick={() => void exportar(`plantas-${e.nombre.replace(/\W+/g, '-')}.csv`, aCsv(e), 'csv')} className={`${boton} flex-1`}>
          {t('pl.csv')}
        </button>
        <button
          type="button"
          onClick={() => window.confirm(`${t('pl.borrar')}: ${e.nombre}`) && void borrar('experimentos', e.id).then(alCambiar)}
          className={`${boton} border-red-400/40 text-red-200`}
        >
          {t('borrar')}
        </button>
      </div>
    </section>
  );
}

/** Altura frente a días: control con línea discontinua y sonido con línea llena. */
function Grafica({ e, etiquetas, vacio }: { e: Experimento; etiquetas: [string, string]; vacio: string }) {
  const puntos = e.mediciones.filter((m) => m.control !== null || m.sonido !== null).sort((a, b) => a.fecha - b.fecha);
  if (puntos.length < 2) return <p className="mt-3 text-sm text-tenue">{vacio}</p>;
  const W = 320;
  const H = 140;
  const M = { izq: 30, der: 8, arr: 8, aba: 20 };
  const dias = puntos.map((m) => (m.fecha - e.inicio) / 86_400_000);
  const valores = puntos.flatMap((m) => [m.control, m.sonido]).filter((v): v is number => v !== null);
  const [d0, d1] = [Math.min(...dias), Math.max(...dias, Math.min(...dias) + 1)];
  const [v0, v1] = [Math.min(...valores, 0), Math.max(...valores) * 1.1 || 1];
  const x = (d: number) => M.izq + ((d - d0) / (d1 - d0)) * (W - M.izq - M.der);
  const y = (v: number) => H - M.aba - ((v - v0) / (v1 - v0)) * (H - M.arr - M.aba);
  const linea = (campo: 'control' | 'sonido') =>
    puntos
      .map((m, i) => (m[campo] === null ? null : `${x(dias[i]).toFixed(1)},${y(m[campo]!).toFixed(1)}`))
      .filter(Boolean)
      .join(' ');
  return (
    <figure className="mt-3">
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full" role="img" aria-label={`${etiquetas[0]} / ${etiquetas[1]}`}>
        <line x1={M.izq} y1={H - M.aba} x2={W - M.der} y2={H - M.aba} stroke="rgb(44,54,86)" />
        <line x1={M.izq} y1={M.arr} x2={M.izq} y2={H - M.aba} stroke="rgb(44,54,86)" />
        <text x={M.izq - 4} y={y(v1) + 8} textAnchor="end" fontSize="9" fill="rgb(148,160,190)">
          {v1.toFixed(0)}
        </text>
        <text x={M.izq - 4} y={y(v0)} textAnchor="end" fontSize="9" fill="rgb(148,160,190)">
          {v0.toFixed(0)}
        </text>
        <text x={x(d0)} y={H - 6} fontSize="9" fill="rgb(148,160,190)">
          {Math.round(d0)}
        </text>
        <text x={x(d1)} y={H - 6} textAnchor="end" fontSize="9" fill="rgb(148,160,190)">
          {Math.round(d1)}
        </text>
        <polyline points={linea('control')} fill="none" stroke="rgb(148,160,190)" strokeWidth="2" strokeDasharray="5 4" />
        <polyline points={linea('sonido')} fill="none" stroke="rgb(176,150,255)" strokeWidth="2.5" />
      </svg>
      <figcaption className="flex gap-4 text-xs text-tenue">
        <span>┄ {etiquetas[0]}</span>
        <span className="text-exploracion">━ {etiquetas[1]}</span>
      </figcaption>
    </figure>
  );
}
