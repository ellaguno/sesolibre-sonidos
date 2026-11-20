import { useState } from 'react';

import { traducir } from '../../core/i18n';
import type { Estado } from '../../core/motor';
import type { Entrada, Idioma } from '../../core/recetas';
import { Cimatica } from './Cimatica';
import { Plantas } from './Plantas';

interface Props {
  idioma: Idioma;
  estado: Estado;
  recetas: Entrada[];
  alVolver: () => void;
}

/** El laboratorio: cimática y plantas. Exploración honesta, con instrucciones y control. */
export function Laboratorio({ idioma, estado, recetas, alVolver }: Props) {
  const t = (c: string) => traducir(idioma, c);
  const [pestana, setPestana] = useState<'cimatica' | 'plantas'>('cimatica');
  const ficha = (activa: boolean) =>
    `min-h-11 flex-1 rounded-full border px-4 text-sm font-semibold ${activa ? 'border-transparent bg-exploracion text-fondo' : 'border-borde text-tenue'}`;
  return (
    <div>
      <header className="flex items-center gap-3">
        <button type="button" onClick={alVolver} className="min-h-11 rounded-full border border-borde px-4 text-sm font-medium text-tenue">
          ← {t('lab.volver')}
        </button>
        <h1 className="text-2xl font-semibold tracking-tight text-exploracion">{t('lab')}</h1>
      </header>
      <div className="mt-4 flex gap-2" role="tablist">
        <button type="button" role="tab" aria-selected={pestana === 'cimatica'} onClick={() => setPestana('cimatica')} className={ficha(pestana === 'cimatica')}>
          {t('cim.titulo')}
        </button>
        <button type="button" role="tab" aria-selected={pestana === 'plantas'} onClick={() => setPestana('plantas')} className={ficha(pestana === 'plantas')}>
          {t('pl.titulo')}
        </button>
      </div>
      <div className="mt-5">{pestana === 'cimatica' ? <Cimatica idioma={idioma} estado={estado} /> : <Plantas idioma={idioma} recetas={recetas} />}</div>
    </div>
  );
}
