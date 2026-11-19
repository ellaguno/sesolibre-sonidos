import { describe, expect, it } from 'vitest';

import { minutosLegibles, reloj, traducir } from '../src/core/i18n';
import { RECETAS, SALAS, duraciones } from '../src/core/recetas';

describe('recetas', () => {
  it('se cargan todas y cada sala tiene alguna', () => {
    expect(RECETAS.length).toBeGreaterThanOrEqual(8);
    for (const sala of SALAS) expect(RECETAS.some((e) => e.receta.sala === sala)).toBe(true);
  });

  it('cada una tiene textos en los dos idiomas y su línea de ciencia', () => {
    for (const { receta: r } of RECETAS) {
      for (const idioma of ['es', 'en'] as const) {
        expect(r.nombre[idioma], r.id).toBeTruthy();
        expect(r.descripcion[idioma], r.id).toBeTruthy();
        expect(r.ciencia[idioma], r.id).toBeTruthy();
      }
    }
  });

  it('la interfaz sabe nombrar cada situación y cada advertencia', () => {
    for (const { receta: r } of RECETAS) {
      for (const idioma of ['es', 'en'] as const) {
        expect(traducir(idioma, `situacion.${r.situacion}`)).not.toBe(`situacion.${r.situacion}`);
        for (const a of r.advertencias) expect(traducir(idioma, `adv.${a}`), `${r.id}: ${a}`).not.toBe(`adv.${a}`);
      }
    }
  });

  it('ofrece la duración propia de la receta entre las opciones', () => {
    for (const { receta: r } of RECETAS) expect(duraciones(r)).toContain(Math.round(r.duracion_s / 60));
  });
});

describe('formato', () => {
  it('reloj y minutos', () => {
    expect(reloj(65)).toBe('1:05');
    expect(reloj(3725)).toBe('1:02:05');
    expect(reloj(-3)).toBe('0:00');
    expect(minutosLegibles('es', 45)).toBe('45 min');
    expect(minutosLegibles('es', 480)).toBe('8 h');
  });
});
