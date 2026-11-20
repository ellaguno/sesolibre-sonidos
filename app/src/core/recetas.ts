// Las recetas del repositorio (../recetas/*.json): el mismo JSON que valida y toca el
// núcleo. Aquí solo se leen para mostrarlas; al tocar, se manda el JSON tal cual.

export type Sala = 'comprobado' | 'prometedor' | 'exploracion' | 'propio';
export type Idioma = 'es' | 'en';
export interface Texto {
  es: string;
  en: string;
}

export interface Receta {
  version: number;
  id: string;
  sala: Sala;
  situacion: string;
  nombre: Texto;
  descripcion: Texto;
  ciencia: Texto;
  fuentes: string[];
  duracion_s: number;
  advertencias: string[];
  capas: Record<string, unknown>[];
  /** Solo en los diseños propios: lo que eligió el usuario, para poder editarlo. */
  diseno?: unknown;
  [otro: string]: unknown;
}

export interface Entrada {
  receta: Receta;
  /** El JSON original, que es lo que recibe el núcleo. */
  json: string;
}

const crudos = import.meta.glob('../../../recetas/*.json', { eager: true, query: '?raw', import: 'default' }) as Record<
  string,
  string
>;

/** Las salas del catálogo. Los diseños propios van aparte, arriba. */
export const SALAS: Sala[] = ['comprobado', 'prometedor', 'exploracion'];

export const RECETAS: Entrada[] = Object.keys(crudos)
  .sort()
  .map((ruta) => ({ receta: JSON.parse(crudos[ruta]) as Receta, json: crudos[ruta] }));

/** Las recetas del catálogo de una sala (las del laboratorio se usan desde su pantalla). */
export function deSala(sala: Sala): Entrada[] {
  return RECETAS.filter((e) => e.receta.sala === sala && e.receta.situacion !== 'laboratorio');
}

/** Minutos que se ofrecen para una receta: los habituales más la duración propia. */
export function duraciones(receta: Receta): number[] {
  const propia = Math.round(receta.duracion_s / 60);
  return [...new Set([10, 20, 30, 45, 60, 90, 480, propia])].sort((a, b) => a - b);
}

export function necesitaAuriculares(receta: Receta): boolean {
  return receta.advertencias.includes('auriculares');
}
