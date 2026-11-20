// El motor de sonido visto desde la web. En Android es el plugin nativo `Sonidos`
// (app/plugin): un servicio de reproducción que sigue con la pantalla apagada. En el
// navegador es una simulación sin sonido (el núcleo en WebAssembly llega en la Fase 3).

import { registerPlugin, type PluginListenerHandle } from '@capacitor/core';

export interface Estado {
  /** Hay una receta cargada (sonando o en pausa). */
  activo: boolean;
  sonando: boolean;
  cargando: boolean;
  terminado: boolean;
  recetaId: string | null;
  segundos: number;
  duracion: number;
  auriculares: boolean;
  /** true en el navegador: el reloj corre pero no hay sonido. */
  simulado: boolean;
}

export const EN_REPOSO: Estado = {
  activo: false,
  sonando: false,
  cargando: false,
  terminado: false,
  recetaId: null,
  segundos: 0,
  duracion: 0,
  auriculares: false,
  simulado: false,
};

export interface OpcionesTocar {
  /** El JSON de la receta, tal cual está en recetas/. */
  receta: string;
  recetaId: string;
  titulo: string;
  subtitulo: string;
  /** Temporizador; si falta, la duración de la receta. */
  minutos?: number;
  /** Volumen del usuario, 0 o menos (el modo noche). */
  volumenDb?: number;
}

export interface SonidosPlugin {
  tocar(o: OpcionesTocar): Promise<void>;
  pausar(): Promise<void>;
  reanudar(): Promise<void>;
  parar(): Promise<void>;
  extender(o: { minutos: number }): Promise<{ minutos: number }>;
  volumen(o: { db: number }): Promise<void>;
  /** Espectro de lo que suena: valores 0..1, graves primero. Vacío si no suena nada. */
  espectro(o: { bandas: number }): Promise<{ bandas: number[] }>;
  /** `motivo` nulo si el núcleo acepta la receta. */
  validar(o: { receta: string }): Promise<{ motivo: string | null }>;
  estado(): Promise<Estado>;
  addListener(evento: 'estado', oyente: (e: Estado) => void): Promise<PluginListenerHandle>;
  addListener(evento: 'error', oyente: (e: { mensaje: string }) => void): Promise<PluginListenerHandle>;
}

// Una sola instancia: Capacitor llama a esta fábrica una vez por cada orden que llega
// antes de que termine la primera carga, y los oyentes deben quedar todos en la misma.
let web: Promise<unknown> | null = null;

export const Sonidos = registerPlugin<SonidosPlugin>('Sonidos', {
  web: () => (web ??= import('./motor-web').then((m) => new m.SonidosWeb())),
});

/** El modo noche baja todo 8 dB. Nunca se sube por encima de la receta. */
export const NOCHE_DB = -8;
