import { describe, expect, it } from 'vitest';

import { DISENO_INICIAL, armonicosAudibles, codificar, construir, copiaDe, decodificar, type Diseno } from '../src/core/diseno';
import { RECETAS } from '../src/core/recetas';

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
    expect(codigo.startsWith('sonidos2:')).toBe(true);
    expect(codigo.length).toBeLessThan(260);
    const vuelta = decodificar(`mira esto: ${codigo} ¡pruébalo!`)!;
    expect(vuelta.capas).toEqual(r.capas);
    expect(vuelta.nombre.es).toBe('Mañana de niebla ☁');
    expect(vuelta.sala).toBe('propio');
    expect(vuelta.id).not.toBe('');
    expect(vuelta.diseno).toBeTruthy(); // se puede seguir editando
    // Un catálogo disfrazado no se cuela como "comprobado".
    // Una receta que no nació en el diseñador viaja en el formato largo, y entra igual.
    const larga = codificar({ ...r, diseno: undefined, sala: 'comprobado', ciencia: { es: 'cura todo', en: 'x' } });
    expect(larga.startsWith('sonidos1:')).toBe(true);
    const falso = decodificar(larga)!;
    expect(falso.sala).toBe('propio');
    expect(falso.ciencia.es).not.toContain('cura');
  });

  it('rechaza lo que no es un código', () => {
    expect(decodificar('hola')).toBeNull();
    expect(decodificar('sonidos1:AAAA')).toBeNull();
    expect(decodificar('sonidos2:AAAA')).toBeNull();
  });

  it('un código corto con valores inventados cae en los valores seguros', () => {
    const raro = 'sonidos2:' + btoa(JSON.stringify(['x', 'mar.inventado', 'bomba', 'q', 999, 'arpa', 'z', 'binaural', 1e9, -5, 1, 'gong', 30]));
    const r = decodificar(raro)!;
    expect(r.sala).toBe('propio');
    const pulso = r.capas.find((c) => c.tipo === 'pulso')!;
    expect(pulso.portadora_hz).toBe(1000);
    expect(pulso.beat_hz).toBe(0.5);
    expect(r.capas.some((c) => c.ambiente === 'hidrofono')).toBe(true);
  });

  it('la nota del colchón y el volumen por parte llegan a la receta y viajan en el código', () => {
    const d: Diseno = { ...DISENO_INICIAL, fondos: ['lluvia'], instrumento: 'arpa', nota: 132, volumen: { fondo: -6, colchon: 6, instrumento: 6 } };
    const r = construir(d);
    const [lluvia, colchon, arpa] = r.capas;
    expect(lluvia.nivel_db).toBe(-25); // -19 de fábrica, 6 menos
    expect(colchon.nivel_db).toBe(-11);
    expect(colchon.raiz_hz).toBe(132);
    expect(arpa.raiz_hz).toBe(132);
    expect(arpa.nivel_db).toBe(-17);
    const codigo = codificar(r);
    expect(codigo.startsWith('sonidos2:')).toBe(true);
    expect(decodificar(codigo)!.capas).toEqual(r.capas);
    // Valores fuera de rango: la nota se acota y ningún nivel pasa de 0 dB.
    const bruto = construir({ ...d, nota: 99999, volumen: { colchon: 999 } });
    expect(bruto.capas[1].raiz_hz).toBe(400);
    expect(bruto.capas.every((c) => (c.nivel_db as number) <= 0)).toBe(true);
  });

  it('un diseño guardado antes de existir estos campos se lee igual', () => {
    const viejo = 'sonidos2:' + btoa(JSON.stringify(['v', 'lluvia', 'cuerdas', 'menor', 440, 'ninguno', 'pocas', 'ninguno', 200, 6, 0, 'ninguno', 30]));
    const r = decodificar(viejo)!;
    expect(r.capas.find((c) => c.tipo === 'colchon')!.raiz_hz).toBeUndefined();
    expect(r.capas.find((c) => c.tipo === 'colchon')!.nivel_db).toBe(-17);
  });

  it('usar como base copia una receta del catálogo tal cual, como diseño propio', () => {
    const origen = RECETAS.find((e) => e.receta.id === 'exploracion.schumann')!.receta;
    const copia = copiaDe(origen, '(copia)');
    expect(copia.sala).toBe('propio');
    expect(copia.capas).toEqual(origen.capas);
    expect(copia.nombre.es).toContain('(copia)');
    expect(copia.id).not.toBe(origen.id);
    // No nació en el diseñador: viaja en el formato largo y vuelve con las mismas capas.
    const codigo = codificar(copia);
    expect(codigo.startsWith('sonidos1:')).toBe(true);
    expect(decodificar(codigo)!.capas).toEqual(origen.capas);
  });
});
