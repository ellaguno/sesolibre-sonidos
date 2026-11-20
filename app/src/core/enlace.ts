// Enlaces directos: otra app (Somnia, Mind, un atajo) abre una receta con
//   sesolibre-sonidos://tocar?receta=dormir.mar-cuerdas&min=30
// y en la web sirve la misma consulta: https://…/?receta=dormir.mar-cuerdas&min=30

export interface Enlace {
  recetaId: string;
  minutos?: number;
}

export function leerEnlace(url: string): Enlace | null {
  try {
    const q = new URL(url, 'https://x.invalid/').searchParams;
    const recetaId = q.get('receta');
    if (!recetaId || !/^[a-z0-9.-]{1,80}$/i.test(recetaId)) return null;
    const min = Number(q.get('min'));
    return { recetaId, ...(Number.isFinite(min) && min >= 1 && min <= 720 ? { minutos: Math.round(min) } : {}) };
  } catch {
    return null;
  }
}
