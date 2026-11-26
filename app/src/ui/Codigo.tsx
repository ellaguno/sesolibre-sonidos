import { useState } from 'react';

import { traducir } from '../core/i18n';
import type { Idioma } from '../core/recetas';

interface Props {
  idioma: Idioma;
  /** Con código: se muestra para copiarlo. Sin él: se pide uno para importar. */
  codigo?: string;
  alCerrar: () => void;
  alImportar?: (texto: string) => void;
}

/**
 * Ventana propia para mostrar o pedir el código de un diseño. Sustituye al `prompt` del
 * sistema, que con un texto largo empujaba el botón de aceptar fuera de la pantalla: aquí
 * el texto va en una caja de altura fija que se desplaza, y los botones siempre se ven.
 */
export function Codigo({ idioma, codigo, alCerrar, alImportar }: Props) {
  const t = (c: string) => traducir(idioma, c);
  const [texto, setTexto] = useState(codigo ?? '');
  const [nota, setNota] = useState<string | null>(null);
  const importando = codigo === undefined;

  const copiar = async () => {
    try {
      await navigator.clipboard.writeText(texto);
      setNota(t('copiado'));
    } catch {
      setNota(t('copiaAMano'));
    }
  };
  const pegar = async () => {
    try {
      setTexto(await navigator.clipboard.readText());
    } catch {
      setNota(t('pegaAMano'));
    }
  };

  const boton = 'min-h-12 flex-1 rounded-full border border-borde px-4 font-medium';
  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-black/70 p-4" onClick={alCerrar}>
      <div role="dialog" aria-modal="true" aria-label={t(importando ? 'importar' : 'compartir')} onClick={(e) => e.stopPropagation()} className="w-full max-w-md rounded-3xl border border-borde bg-panel p-5">
        <h2 className="text-xl font-semibold text-propio">{t(importando ? 'importar' : 'compartir')}</h2>
        <p className="mt-1 text-sm text-tenue">{t(importando ? 'importarQue' : 'compartirQue')}</p>
        <textarea
          value={texto}
          readOnly={!importando}
          onChange={(e) => setTexto(e.target.value)}
          onFocus={(e) => !importando && e.target.select()}
          rows={4}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          aria-label={t('codigo')}
          className="mt-3 block h-28 w-full resize-none overflow-y-auto break-all rounded-xl border border-borde bg-fondo p-3 font-mono text-xs text-texto"
        />
        {nota && <p className="mt-2 text-sm text-propio">{nota}</p>}
        <div className="mt-4 flex gap-2">
          {importando ? (
            <>
              <button type="button" onClick={() => void pegar()} className={boton}>
                {t('pegar')}
              </button>
              <button type="button" disabled={!texto.trim()} onClick={() => alImportar?.(texto)} className="min-h-12 flex-1 rounded-full bg-propio px-4 font-semibold text-fondo disabled:opacity-40">
                {t('importar')}
              </button>
            </>
          ) : (
            <button type="button" onClick={() => void copiar()} className="min-h-12 flex-1 rounded-full bg-propio px-4 font-semibold text-fondo">
              {t('copiar')}
            </button>
          )}
          <button type="button" onClick={alCerrar} className={boton}>
            {t('cerrar')}
          </button>
        </div>
      </div>
    </div>
  );
}
