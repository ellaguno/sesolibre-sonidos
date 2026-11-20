import { describe, expect, it } from 'vitest';

import { DISENO_INICIAL, armonicosAudibles, codificar, construir, decodificar, type Diseno } from '../src/core/diseno';

describe('diseños', () => {
  it('el diseño inicial da una receta propia con capas', () => {
    const r = construir(DISENO_INICIAL, 'propio.x');
    expect(r.sala).toBe('propio');
    expect(r.capas.length).toBeGreaterThan(0);
    expect(r.techo_db).toBe(-14);
    expect(r.nombre.es).toBe('Mi diseño');
  });

  it('un binaural lleva sus advertencias y valores dentro de los límites del núcleo', () => {
    const d: Diseno = { ...DISENO_INICIAL, frecuencia: 'binaural', hz: 5, beat: 900 };
    const r = construir(d);
    const pulso = r.capas.find((c) => c.tipo === 'pulso')!;
    expect(pulso.portadora_hz).toBe(60);
    expect(pulso.beat_hz).toBe(45);
    expect(r.advertencias).toEqual(['auriculares', 'no_conducir']);
  });

  it('Schumann: los armónicos de 7,83 Hz caen donde se oyen', () => {
    const n = armonicosAudibles(7.83);
    expect(n).toEqual([15, 23, 30, 45]);
    for (const k of n) expect(7.83 * k).toBeGreaterThan(100);
  });

  it('compartir e importar devuelve el mismo sonido, siempre como diseño propio', () => {
    const r = construir({ ...DISENO_INICIAL, nombre: 'Mañana de niebla ☁', fondos: ['lluvia', 'aves'] });
    const codigo = codificar(r);
    expect(codigo.startsWith('sonidos1:')).toBe(true);
    const vuelta = decodificar(`mira esto: ${codigo} ¡pruébalo!`)!;
    expect(vuelta.capas).toEqual(r.capas);
    expect(vuelta.nombre.es).toBe('Mañana de niebla ☁');
    expect(vuelta.sala).toBe('propio');
    expect(vuelta.id).not.toBe('');
    // Un catálogo disfrazado no se cuela como "comprobado".
    const falso = decodificar(codificar({ ...r, sala: 'comprobado', ciencia: { es: 'cura todo', en: 'x' } }))!;
    expect(falso.sala).toBe('propio');
    expect(falso.ciencia.es).not.toContain('cura');
  });

  it('rechaza lo que no es un código', () => {
    expect(decodificar('hola')).toBeNull();
    expect(decodificar('sonidos1:AAAA')).toBeNull();
  });
});
