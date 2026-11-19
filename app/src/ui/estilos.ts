import type { Sala } from '../core/recetas';

/** Clases completas por sala (Tailwind no ve nombres armados a trozos). */
export const DE_SALA: Record<Sala, { texto: string; borde: string; fondo: string; boton: string; barra: string }> = {
  comprobado: {
    texto: 'text-comprobado',
    borde: 'border-comprobado/30',
    fondo: 'bg-comprobado/10',
    boton: 'bg-comprobado text-fondo',
    barra: 'bg-comprobado',
  },
  prometedor: {
    texto: 'text-prometedor',
    borde: 'border-prometedor/30',
    fondo: 'bg-prometedor/10',
    boton: 'bg-prometedor text-fondo',
    barra: 'bg-prometedor',
  },
  exploracion: {
    texto: 'text-exploracion',
    borde: 'border-exploracion/40 border-dashed',
    fondo: 'bg-exploracion/10',
    boton: 'bg-exploracion text-fondo',
    barra: 'bg-exploracion',
  },
};
