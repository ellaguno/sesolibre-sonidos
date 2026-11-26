import { useCallback, useEffect, useRef, useState } from 'react';

import { App as Aplicacion } from '@capacitor/app';
import { Capacitor } from '@capacitor/core';
import { Share } from '@capacitor/share';

import { CREDITOS } from './core/creditos';
import { DISENO_INICIAL, cargarDisenos, codificar, construir, decodificar, entradaDe, guardarDisenos, type Diseno } from './core/diseno';
import { leerEnlace } from './core/enlace';
import { idiomaInicial, traducir } from './core/i18n';
import { NOCHE_DB, Sonidos } from './core/motor';
import { RECETAS, SALAS, deSala, type Entrada, type Idioma } from './core/recetas';
import { useMotor } from './core/useMotor';
import { DE_SALA } from './ui/estilos';
import { Codigo } from './ui/Codigo';
import { Disenador } from './ui/Disenador';
import { Ficha } from './ui/Ficha';
import { Laboratorio } from './ui/lab/Laboratorio';
import { Luna, MarcaSala, Mas } from './ui/iconos';
import { Reproductor } from './ui/Reproductor';
import { Tarjeta } from './ui/Tarjeta';

function guardado(clave: string): string | null {
  try {
    return localStorage.getItem(clave);
  } catch {
    return null;
  }
}

function guardar(clave: string, valor: string): void {
  try {
    localStorage.setItem(clave, valor);
  } catch {
    /* sin almacenamiento: el ajuste dura lo que dure la sesión */
  }
}

export default function App() {
  const [idioma, setIdioma] = useState<Idioma>(idiomaInicial);
  const [noche, setNoche] = useState(() => guardado('sonidos.noche') === '1');
  const [abierta, setAbierta] = useState<Entrada | null>(null);
  const { estado, segundos, error, limpiarError } = useMotor();
  const [fallo, setFallo] = useState<string | null>(null);
  const [disenos, setDisenos] = useState<Entrada[]>(cargarDisenos);
  /** El diseñador abierto: un diseño nuevo o el id del que se edita. */
  const [disenando, setDisenando] = useState<{ diseno: Diseno; id?: string } | null>(null);
  const [nota, setNota] = useState<string | null>(null);
  const [enLaboratorio, setEnLaboratorio] = useState(false);

  const ponerDisenos = (lista: Entrada[]) => {
    setDisenos(lista);
    guardarDisenos(lista);
  };

  const guardarDiseno = async (d: Diseno, id?: string) => {
    const entrada = entradaDe(construir(d, id));
    const { motivo } = await Sonidos.validar({ receta: entrada.json });
    if (motivo) return setFallo(motivo);
    ponerDisenos(id ? disenos.map((e) => (e.receta.id === id ? entrada : e)) : [entrada, ...disenos]);
    setDisenando(null);
    setAbierta(entrada);
  };

  /** La ventana del código: para copiar uno (compartir) o para pegar uno (importar). */
  const [ventanaCodigo, setVentanaCodigo] = useState<{ codigo?: string } | null>(null);

  const compartir = async (entrada: Entrada) => {
    const codigo = codificar(entrada.receta);
    try {
      await Share.share({ title: entrada.receta.nombre[idioma], text: codigo });
    } catch {
      // Sin hoja de compartir (navegador de escritorio) o cancelado: se muestra para copiar.
      setVentanaCodigo({ codigo });
    }
  };

  const importar = async (texto: string) => {
    setVentanaCodigo(null);
    const receta = decodificar(texto);
    if (!receta) return setFallo(traducir(idioma, 'importarMal'));
    const entrada = entradaDe(receta);
    const { motivo } = await Sonidos.validar({ receta: entrada.json });
    if (motivo) return setFallo(motivo);
    ponerDisenos([entrada, ...disenos]);
    setAbierta(entrada);
  };
  const t = (c: string) => traducir(idioma, c);

  useEffect(() => {
    document.documentElement.lang = idioma;
    guardar('sonidos.idioma', idioma);
  }, [idioma]);

  useEffect(() => {
    guardar('sonidos.noche', noche ? '1' : '0');
    Sonidos.volumen({ db: noche ? NOCHE_DB : 0 }).catch(() => undefined);
  }, [noche]);

  const tocar = useCallback(
    (entrada: Entrada, minutos: number) => {
      const r = entrada.receta;
      setAbierta(null);
      setFallo(null);
      limpiarError();
      Sonidos.tocar({
        receta: entrada.json,
        recetaId: r.id,
        titulo: r.nombre[idioma],
        subtitulo: `${traducir(idioma, `sala.${r.sala}`)} · SesoLibre Sonidos`,
        minutos,
        volumenDb: noche ? NOCHE_DB : 0,
      }).catch((e: unknown) => setFallo(e instanceof Error ? e.message : String(e)));
    },
    [idioma, noche, limpiarError],
  );

  // Enlaces directos. En Android la receta empieza sola (la app ya está visible, que es lo
  // que el sistema exige); en el navegador se abre su ficha, porque sin un toque no suena.
  const alEnlace = useRef<(url: string) => void>(() => undefined);
  alEnlace.current = (url: string) => {
    const enlace = leerEnlace(url);
    const entrada = enlace && [...disenos, ...RECETAS].find((e) => e.receta.id === enlace.recetaId);
    if (!enlace || !entrada) return;
    if (Capacitor.isNativePlatform()) tocar(entrada, enlace.minutos ?? Math.round(entrada.receta.duracion_s / 60));
    else setAbierta(entrada);
  };
  useEffect(() => {
    if (!Capacitor.isNativePlatform()) {
      alEnlace.current(window.location.href);
      return;
    }
    void Aplicacion.getLaunchUrl().then((u) => u?.url && alEnlace.current(u.url));
    const oyente = Aplicacion.addListener('appUrlOpen', (e) => alEnlace.current(e.url));
    return () => void oyente.then((h) => h.remove());
  }, []);

  const sonando = estado.activo ? ([...disenos, ...RECETAS].find((e) => e.receta.id === estado.recetaId) ?? null) : null;
  const aviso = fallo ?? error;

  const catalogo = [...disenos, ...RECETAS].filter((e) => e.receta.situacion !== 'laboratorio');
  const reproductor = sonando && sonando.receta.situacion !== 'laboratorio' && (
    <Reproductor
      entrada={sonando}
      estado={estado}
      segundos={segundos}
      idioma={idioma}
      alPausar={() => void Sonidos.pausar()}
      alReanudar={() => void Sonidos.reanudar()}
      alParar={() => void Sonidos.parar()}
      alExtender={() => void Sonidos.extender({ minutos: 15 })}
    />
  );

  if (enLaboratorio) {
    return (
      <div className={`mx-auto max-w-xl px-4 pt-[max(1.25rem,env(safe-area-inset-top))] ${reproductor ? 'pb-40' : 'pb-10'}`}>
        {aviso && (
          <p role="alert" className="mb-4 rounded-xl border border-red-400/40 bg-red-400/10 p-3 text-sm text-red-200">
            {aviso}
          </p>
        )}
        <Laboratorio
          idioma={idioma}
          estado={estado}
          recetas={catalogo}
          alVolver={() => {
            setEnLaboratorio(false);
            if (estado.recetaId === 'laboratorio.cimatica') void Sonidos.parar();
            void Sonidos.volumen({ db: noche ? NOCHE_DB : 0 });
          }}
        />
        {reproductor}
      </div>
    );
  }

  return (
    <div className={`mx-auto max-w-xl px-4 pt-[max(1.25rem,env(safe-area-inset-top))] ${sonando ? 'pb-32' : 'pb-10'}`}>
      <header className="flex items-center gap-3">
        <img src={`${import.meta.env.BASE_URL}favicon.svg`} alt="" className="h-12 w-12 shrink-0 rounded-[26%] shadow-lg shadow-black/40" />
        <div className="min-w-0 flex-1">
          <h1 className="font-marca text-[1.75rem] font-medium lowercase leading-none tracking-wide text-[#b7b4ff]" aria-label="SesoLibre Sonidos">
            sesonidos
          </h1>
        </div>
        <button
          type="button"
          onClick={() => setNoche((n) => !n)}
          aria-pressed={noche}
          title={t('nocheQue')}
          className={`flex min-h-11 items-center gap-1.5 rounded-full border px-3 text-sm font-medium transition ${
            noche ? 'border-transparent bg-texto text-fondo' : 'border-borde text-tenue'
          }`}
        >
          <Luna className="h-4 w-4" />
          {t('noche')}
          <span className="tabular-nums opacity-80">{noche ? '−8 dB' : ''}</span>
        </button>
        <button
          type="button"
          onClick={() => setIdioma((i) => (i === 'es' ? 'en' : 'es'))}
          aria-label={idioma === 'es' ? 'Switch to English' : 'Cambiar a español'}
          className="flex h-11 w-11 items-center justify-center rounded-full border border-borde text-sm font-semibold uppercase text-tenue"
        >
          {idioma === 'es' ? 'en' : 'es'}
        </button>
      </header>
      <p className="mt-2 text-tenue">{t('lema')}</p>

      {noche && (
        <p className="mt-3 flex items-start gap-2 rounded-xl border border-borde bg-panel/70 p-3 text-sm text-tenue">
          <Luna className="mt-0.5 h-4 w-4 shrink-0" />
          {t('nocheAyuda')}
        </p>
      )}
      {estado.simulado && (
        <p className="mt-4 rounded-xl border border-borde bg-panel/70 p-3 text-sm text-tenue">{t('simulado')}</p>
      )}
      {aviso && (
        <p role="alert" className="mt-4 rounded-xl border border-red-400/40 bg-red-400/10 p-3 text-sm text-red-200">
          {aviso}
        </p>
      )}

      {nota && (
        <p role="status" className="mt-4 rounded-xl border border-propio/40 bg-propio/10 p-3 text-sm text-propio" onClick={() => setNota(null)}>
          {nota}
        </p>
      )}

      <main>
        <section className="mt-8">
          <h2 className="flex items-center gap-2 text-sm font-semibold uppercase tracking-wider text-propio">
            <MarcaSala sala="propio" className="h-5 w-5" />
            {t('sala.propio')}
          </h2>
          <p className="mt-1 text-sm text-tenue">{t('sala.propio.que')}</p>
          <div className="mt-3 space-y-3">
            {disenos.map((e) => (
              <Tarjeta
                key={e.receta.id}
                entrada={e}
                idioma={idioma}
                sonando={estado.activo && estado.sonando && estado.recetaId === e.receta.id}
                alAbrir={() => setAbierta(e)}
              />
            ))}
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => setDisenando({ diseno: DISENO_INICIAL })}
                className="flex min-h-12 flex-1 items-center justify-center gap-2 rounded-2xl border border-dashed border-propio/50 font-medium text-propio"
              >
                <Mas className="h-5 w-5" />
                {t('disenar')}
              </button>
              <button type="button" onClick={() => setVentanaCodigo({})} className="min-h-12 rounded-2xl border border-borde px-5 font-medium text-tenue">
                {t('importar')}
              </button>
            </div>
          </div>
        </section>

        {SALAS.map((sala) => {
          const recetas = deSala(sala);
          if (recetas.length === 0) return null;
          const s = DE_SALA[sala];
          return (
            <section key={sala} className={sala === 'exploracion' ? 'mt-10 border-t border-dashed border-exploracion/40 pt-8' : 'mt-8'}>
              <h2 className={`flex items-center gap-2 text-sm font-semibold uppercase tracking-wider ${s.texto}`}>
                <MarcaSala sala={sala} className="h-5 w-5" />
                {t(`sala.${sala}`)}
              </h2>
              <p className="mt-1 text-sm text-tenue">{t(`sala.${sala}.que`)}</p>
              <div className="mt-3 space-y-3">
                {recetas.map((e) => (
                  <Tarjeta
                    key={e.receta.id}
                    entrada={e}
                    idioma={idioma}
                    sonando={estado.activo && estado.sonando && estado.recetaId === e.receta.id}
                    alAbrir={() => setAbierta(e)}
                  />
                ))}
                {sala === 'exploracion' && (
                  <button
                    type="button"
                    onClick={() => setEnLaboratorio(true)}
                    className="flex w-full items-center gap-4 rounded-2xl border border-dashed border-exploracion/40 bg-panel/80 p-4 text-left"
                  >
                    <span className="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-exploracion/10 text-2xl text-exploracion" aria-hidden="true">
                      ⚗
                    </span>
                    <span>
                      <span className="block text-lg font-medium">{t('lab')}</span>
                      <span className="block text-sm text-tenue">{t('lab.que')}</span>
                    </span>
                  </button>
                )}
              </div>
            </section>
          );
        })}
      </main>

      <footer className="mt-10 space-y-1 text-center text-sm text-tenue">
        <p>{t('pie')}</p>
        <p>{t('adv.siempre')}</p>
        <p className="pt-3 text-xs">
          <a href="https://sesolibre.com" target="_blank" rel="noopener noreferrer" className="font-semibold text-propio hover:underline">
            SesoLibre.com
          </a>
          <span className="mx-2">·</span>Sonidos v{__APP_VERSION__}
          <span className="mx-2">·</span>
          {t('pieInfo')}
        </p>
        {sonando && <p>{t('apagada')}</p>}
        <details className="pt-4 text-left">
          <summary className="cursor-pointer text-center underline underline-offset-2">{t('creditos')}</summary>
          <ul className="mt-2 space-y-1 text-xs">
            {CREDITOS.map((c) => (
              <li key={c.url}>
                <a href={c.url} target="_blank" rel="noreferrer" className="underline underline-offset-2">
                  {c.que}
                </a>{' '}
                · {c.quien} · {c.licencia}
              </li>
            ))}
          </ul>
        </details>
      </footer>

      {abierta && (
        <Ficha
          entrada={abierta}
          idioma={idioma}
          auriculares={estado.auriculares}
          simulado={estado.simulado}
          alCerrar={() => setAbierta(null)}
          alTocar={(m) => tocar(abierta, m)}
          {...(abierta.receta.sala === 'propio'
            ? {
                alCompartir: () => void compartir(abierta),
                alBorrar: () => {
                  ponerDisenos(disenos.filter((e) => e.receta.id !== abierta.receta.id));
                  setAbierta(null);
                },
                ...(abierta.receta.diseno
                  ? {
                      alEditar: () => {
                        setDisenando({ diseno: abierta.receta.diseno as Diseno, id: abierta.receta.id });
                        setAbierta(null);
                      },
                    }
                  : {}),
              }
            : {})}
        />
      )}

      {ventanaCodigo && (
        <Codigo idioma={idioma} codigo={ventanaCodigo.codigo} alCerrar={() => setVentanaCodigo(null)} alImportar={(texto) => void importar(texto)} />
      )}

      {disenando && (
        <Disenador
          inicial={disenando.diseno}
          idioma={idioma}
          alCerrar={() => setDisenando(null)}
          alGuardar={(d) => void guardarDiseno(d, disenando.id)}
        />
      )}

      {reproductor}
    </div>
  );
}
