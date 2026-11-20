import { describe, expect, it } from 'vitest';

import { leerEnlace } from '../src/core/enlace';

describe('enlaces directos', () => {
  it('lee receta y minutos del esquema propio y de la web', () => {
    expect(leerEnlace('sesolibre-sonidos://tocar?receta=dormir.mar-cuerdas&min=30')).toEqual({ recetaId: 'dormir.mar-cuerdas', minutos: 30 });
    expect(leerEnlace('https://sonidos.example/?receta=respirar.seis-por-minuto')).toEqual({ recetaId: 'respirar.seis-por-minuto' });
  });
  it('ignora lo que no entiende', () => {
    expect(leerEnlace('sesolibre-sonidos://tocar')).toBeNull();
    expect(leerEnlace('sesolibre-sonidos://tocar?receta=<script>')).toBeNull();
    expect(leerEnlace('sesolibre-sonidos://tocar?receta=a.b&min=99999')).toEqual({ recetaId: 'a.b' });
  });
});
