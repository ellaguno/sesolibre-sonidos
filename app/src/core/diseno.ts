// Los diseños del usuario: unas pocas elecciones → una receta que el núcleo valida y toca
// como cualquier otra. Se guardan en el dispositivo y se comparten como un código de texto.

import type { Entrada, Receta } from './recetas';

export const FONDOS = ['mar', 'olas', 'burbujas', 'oleaje', 'gaviotas', 'viento', 'lluvia', 'mirlo', 'verderon', 'mosquitero', 'campanero', 'selva', 'noche', 'grillos', 'ranas', 'lienzo_abismo', 'lienzo_caribe', 'aves', 'rosa', 'marron'] as const;
/** Los fondos, agrupados como se muestran en el diseñador. */
export const GRUPOS_DE_FONDO: { titulo: string; fondos: (typeof FONDOS)[number][] }[] = [
  { titulo: 'd.fondo.agua', fondos: ['mar', 'olas', 'oleaje', 'burbujas'] },
  { titulo: 'd.fondo.aire', fondos: ['viento', 'lluvia'] },
  { titulo: 'd.fondo.aves', fondos: ['mirlo', 'verderon', 'mosquitero', 'campanero', 'selva', 'gaviotas'] },
  { titulo: 'd.fondo.noche', fondos: ['noche', 'grillos', 'ranas'] },
  { titulo: 'd.fondo.lienzo', fondos: ['lienzo_abismo', 'lienzo_caribe'] },
  { titulo: 'd.fondo.ruido', fondos: ['rosa', 'marron', 'aves'] },
];
export const COLCHONES = ['ninguno', 'sintetico', 'cuerdas', 'cello', 'violin', 'flauta', 'corno', 'trombones'] as const;
export const MODOS = ['re_pentatonico', 'menor', 'quintas'] as const;
export const INSTRUMENTOS = ['ninguno', 'arpa', 'vibrafono', 'marimba', 'flauta', 'fagot', 'corno', 'trompeta', 'campanas', 'armonicos'] as const;
export const FRECUENCIAS = ['ninguno', 'tono', 'binaural', 'isocronico', 'serie'] as const;
export const EVENTOS = ['ninguno', 'carillon', 'platillo', 'gong'] as const;
/** Partes del sonido con volumen propio en el diseñador. */
export const PARTES = ['fondo', 'colchon', 'instrumento', 'frecuencia', 'evento', 'respiracion'] as const;
export type Parte = (typeof PARTES)[number];
/** Tres escalones de volumen por parte: 6 dB abajo, el de fábrica, 6 dB arriba. */
export const VOLUMENES = { suave: -6, normal: 0, presente: 6 } as const;
/** Notas de atajo para el colchón (Hz de su nota más grave); 0 = la de siempre (Re). */
export const NOTAS = [0, 108, 132, 62.64] as const;
export const DENSIDADES = { pocas: 3, algunas: 6, muchas: 12 } as const;

export interface Diseno {
  nombre: string;
  fondos: (typeof FONDOS)[number][];
  colchon: (typeof COLCHONES)[number];
  modo: (typeof MODOS)[number];
  afinacion: 440 | 432;
  instrumento: (typeof INSTRUMENTOS)[number];
  densidad: keyof typeof DENSIDADES;
  frecuencia: (typeof FRECUENCIAS)[number];
  /** Tono puro o portadora del pulso, en Hz. */
  hz: number;
  /** Pulso (binaural, isocrónico) o base de la serie de armónicos, en Hz. */
  beat: number;
  respiracion: boolean;
  /** Golpes sueltos y espaciados. Falta en los diseños guardados antes de existir. */
  evento?: (typeof EVENTOS)[number];
  minutos: number;
  /** Nota más grave del colchón y del instrumento, en Hz (40–400). 0 o ausente = Re. */
  nota?: number;
  /** Decibelios de más o de menos por parte (−6, 0 o 6). Ausente = 0. */
  volumen?: Partial<Record<Parte, number>>;
}

export const DISENO_INICIAL: Diseno = {
  nombre: '',
  fondos: ['oleaje'],
  colchon: 'cuerdas',
  modo: 're_pentatonico',
  afinacion: 440,
  instrumento: 'ninguno',
  densidad: 'pocas',
  frecuencia: 'ninguno',
  hz: 200,
  beat: 6,
  respiracion: false,
  evento: 'ninguno',
  minutos: 30,
};

const acotar = (x: number, lo: number, hi: number, def: number) => (Number.isFinite(x) ? Math.min(hi, Math.max(lo, x)) : def);

function capaDeFondo(f: Diseno['fondos'][number]): Record<string, unknown> {
  switch (f) {
    case 'mar':
      return { tipo: 'ambiente', ambiente: 'hidrofono', paso_bajo_hz: 3000, nivel_db: -25 };
    case 'olas':
      return { tipo: 'ambiente', ambiente: 'olas', paso_bajo_hz: 2500, nivel_db: -30 };
    case 'burbujas':
      return { tipo: 'ambiente', ambiente: 'burbujas', paso_bajo_hz: 4000, nivel_db: -31 };
    case 'oleaje':
      return { tipo: 'ruido', color: 'rosa', paso_bajo_hz: 1400, oleaje_s: 11, nivel_db: -17 };
    case 'viento':
      return { tipo: 'naturaleza', clase: 'viento', nivel_db: -23 };
    case 'lluvia':
      return { tipo: 'naturaleza', clase: 'lluvia', nivel_db: -19 };
    case 'aves':
      return { tipo: 'naturaleza', clase: 'aves', nivel_db: -30, reverb: 0.5 };
    case 'mirlo':
    case 'verderon':
    case 'mosquitero':
    case 'campanero':
      return { tipo: 'ambiente', ambiente: f, nivel_db: -30 };
    case 'lienzo_abismo':
    case 'lienzo_caribe':
      return { tipo: 'ambiente', ambiente: f, nivel_db: -19 };
    case 'gaviotas':
      return { tipo: 'ambiente', ambiente: 'gaviotas', nivel_db: -34 };
    case 'selva':
      return { tipo: 'ambiente', ambiente: 'pita', nivel_db: -30 };
    case 'noche':
      return { tipo: 'ambiente', ambiente: 'chotacabras', nivel_db: -30 };
    case 'grillos':
      return { tipo: 'ambiente', ambiente: 'grillos', nivel_db: -33 };
    case 'ranas':
      return { tipo: 'ambiente', ambiente: 'ranas', nivel_db: -33 };
    case 'rosa':
      return { tipo: 'ruido', color: 'rosa', paso_bajo_hz: 1800, nivel_db: -18 };
    case 'marron':
      return { tipo: 'ruido', color: 'marron', nivel_db: -19 };
  }
}

/** Los armónicos de `base` que caen donde se oyen bien (≈110–450 Hz): n, 1,5n, 2n y 3n. */
export function armonicosAudibles(base: number): number[] {
  const n = Math.max(1, Math.ceil(110 / base));
  return [...new Set([n, Math.round(n * 1.5), n * 2, n * 3])].filter((k) => base * k >= 40 && base * k <= 4000);
}

export function construir(d: Diseno, id = `propio.${Date.now().toString(36)}`): Receta {
  const capas: Record<string, unknown>[] = d.fondos.slice(0, 3).map(capaDeFondo);
  const nFondos = capas.length;
  const nota = d.nota && d.nota > 0 ? acotar(d.nota, 40, 400, 0) : 0;
  const armonia = { modo: d.modo, afinacion_hz: d.afinacion, ciclo_s: 200, ...(nota ? { raiz_hz: nota } : {}) };
  if (d.colchon !== 'ninguno') {
    capas.push({
      tipo: 'colchon',
      ...armonia,
      voces: 5,
      nivel_db: -17,
      ...(d.colchon === 'sintetico' ? { brillo: 0.4 } : { timbre: d.colchon, reverb: 0.5 }),
    });
  }
  if (d.instrumento !== 'ninguno') {
    capas.push({
      tipo: 'instrumento',
      instrumento: d.instrumento,
      ...armonia,
      notas_por_minuto: DENSIDADES[d.densidad],
      octavas: ['corno', 'fagot'].includes(d.instrumento) ? [0, 1] : ['flauta', 'armonicos', 'campanas'].includes(d.instrumento) ? [2, 2] : [1, 2],
      nivel_db: ['flauta', 'armonicos', 'corno', 'trompeta'].includes(d.instrumento) ? -29 : d.instrumento === 'campanas' ? -27 : -23,
      reverb: 0.7,
      desde_s: 10,
    });
  }
  const advertencias: string[] = [];
  const hz = acotar(d.hz, 60, 1000, 200);
  if (d.frecuencia === 'tono') {
    capas.push({ tipo: 'tono', hz: acotar(d.hz, 40, 4000, 432), armonicos: 2, nivel_db: -27 });
  } else if (d.frecuencia === 'binaural' || d.frecuencia === 'isocronico') {
    capas.push({
      tipo: 'pulso',
      forma: d.frecuencia,
      portadora_hz: hz,
      beat_hz: acotar(d.beat, 0.5, 45, 6),
      nivel_db: d.frecuencia === 'binaural' ? -32 : -31,
      desde_s: 10,
      transicion_s: 20,
    });
    advertencias.push(...(d.frecuencia === 'binaural' ? ['auriculares', 'no_conducir'] : ['no_conducir']));
  } else if (d.frecuencia === 'serie') {
    const base = acotar(d.beat, 1, 200, 7.83);
    capas.push({ tipo: 'serie', base_hz: base, armonicos: armonicosAudibles(base), nivel_db: -28 });
  }
  if (d.evento && d.evento !== 'ninguno') {
    capas.push({ tipo: 'eventos', evento: d.evento, cada_s: d.evento === 'gong' ? 120 : 60, nivel_db: d.evento === 'gong' ? -30 : -27, reverb: 0.8, desde_s: 5 });
  }
  if (d.respiracion) {
    capas.push({ tipo: 'respiracion', inhalar_s: 4, exhalar_s: 6, hz: 196, nivel_db: -17, transicion_s: 2 });
  }
  // El volumen de cada parte, sobre el nivel de fábrica de su capa. Nunca por encima de 0 dB
  // (el núcleo lo rechazaría) y siempre bajo el mismo techo que el resto de la app.
  const parteDe = (capa: Record<string, unknown>, i: number): Parte =>
    i < nFondos ? 'fondo' : ({ colchon: 'colchon', instrumento: 'instrumento', eventos: 'evento', respiracion: 'respiracion' } as Record<string, Parte>)[capa.tipo as string] ?? 'frecuencia';
  capas.forEach((capa, i) => {
    const extra = acotar(d.volumen?.[parteDe(capa, i)] ?? 0, -6, 6, 0);
    if (extra) capa.nivel_db = Math.min(0, (capa.nivel_db as number) + extra);
  });
  const nombre = d.nombre.trim() || 'Mi diseño';
  const minutos = acotar(d.minutos, 1, 720, 30);
  return {
    version: 1,
    id,
    sala: 'propio',
    situacion: 'propio',
    nombre: { es: nombre, en: nombre },
    descripcion: { es: '', en: '' },
    ciencia: {
      es: 'Diseño personal: nadie lo ha evaluado.',
      en: 'Personal design: nobody has evaluated it.',
    },
    fuentes: [],
    duracion_s: minutos * 60,
    fade_in_s: 8,
    fade_out_s: Math.min(60, minutos * 10),
    semilla: Date.now() % 1_000_000,
    techo_db: -14,
    advertencias,
    espacio: { rt60_s: 6, amortiguacion_hz: 3500 },
    capas,
    diseno: d,
  };
}

/** Una receta del catálogo copiada a "Tus diseños": suena igual, y ya es del usuario. */
export function copiaDe(receta: Receta, sufijo: string): Receta {
  const nombre = (idioma: 'es' | 'en') => `${receta.nombre[idioma]} ${sufijo}`.slice(0, 60);
  return {
    ...receta,
    id: `propio.${Date.now().toString(36)}`,
    sala: 'propio',
    situacion: 'propio',
    nombre: { es: nombre('es'), en: nombre('en') },
    diseno: undefined,
  };
}

export function entradaDe(receta: Receta): Entrada {
  return { receta, json: JSON.stringify(receta) };
}

// ------------------------------------------------------------------ guardar

const CLAVE = 'sonidos.disenos';

export function cargarDisenos(): Entrada[] {
  try {
    const crudo = JSON.parse(localStorage.getItem(CLAVE) ?? '[]') as Receta[];
    return crudo.filter((r) => r && r.version === 1 && Array.isArray(r.capas)).map(entradaDe);
  } catch {
    return [];
  }
}

export function guardarDisenos(lista: Entrada[]): void {
  try {
    localStorage.setItem(CLAVE, JSON.stringify(lista.map((e) => e.receta)));
  } catch {
    /* sin almacenamiento: duran lo que dure la sesión */
  }
}

// ------------------------------------------------------------------ compartir

// Dos formatos. El corto (`sonidos2:`) lleva solo lo que el usuario eligió en el diseñador,
// unas 150 letras, y la receta se reconstruye al importar. El largo (`sonidos1:`) lleva la
// receta entera; se sigue leyendo, y se usa para recetas que no nacieron en el diseñador.
const PREFIJO = 'sonidos1:';
const PREFIJO_CORTO = 'sonidos2:';

const a64 = (texto: string) => {
  let bin = '';
  new TextEncoder().encode(texto).forEach((b) => (bin += String.fromCharCode(b)));
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
};
const de64 = (b64: string) =>
  new TextDecoder().decode(Uint8Array.from(atob(b64.replace(/-/g, '+').replace(/_/g, '/')), (c) => c.charCodeAt(0)));

/** El diseño en una lista corta y posicional. */
function compacto(d: Diseno): unknown[] {
  return [d.nombre.trim().slice(0, 60), d.fondos.join('.'), d.colchon, d.modo, d.afinacion, d.instrumento, d.densidad, d.frecuencia, d.hz, d.beat, d.respiracion ? 1 : 0, d.evento ?? 'ninguno', d.minutos,
    d.nota ?? 0, PARTES.map((p) => d.volumen?.[p] ?? 0).join('.')];
}

/** Lo contrario, sin fiarse de nada: lo que no reconoce vuelve al valor inicial. */
function deCompacto(v: unknown): Diseno | null {
  if (!Array.isArray(v) || v.length < 8) return null;
  const en = <T extends string>(lista: readonly T[], x: unknown, def: T): T => (lista.includes(x as T) ? (x as T) : def);
  const num = (x: unknown, def: number) => (typeof x === 'number' && Number.isFinite(x) ? x : def);
  const I = DISENO_INICIAL;
  return {
    nombre: String(v[0] ?? '').slice(0, 60),
    fondos: String(v[1] ?? '').split('.').filter((f): f is Diseno['fondos'][number] => (FONDOS as readonly string[]).includes(f)).slice(0, 3),
    colchon: en(COLCHONES, v[2], I.colchon),
    modo: en(MODOS, v[3], I.modo),
    afinacion: v[4] === 432 ? 432 : 440,
    instrumento: en(INSTRUMENTOS, v[5], I.instrumento),
    densidad: en(Object.keys(DENSIDADES) as (keyof typeof DENSIDADES)[], v[6], I.densidad),
    frecuencia: en(FRECUENCIAS, v[7], I.frecuencia),
    hz: num(v[8], I.hz),
    beat: num(v[9], I.beat),
    respiracion: v[10] === 1,
    evento: en(EVENTOS, v[11], 'ninguno'),
    minutos: num(v[12], I.minutos),
    nota: num(v[13], 0),
    volumen: Object.fromEntries(
      String(v[14] ?? '').split('.').map((x, i) => [PARTES[i], [-6, 6].includes(Number(x)) ? Number(x) : 0]).filter(([p, x]) => p && x !== 0),
    ) as Diseno['volumen'],
  };
}

/** Un diseño como texto que cabe en un mensaje. */
export function codificar(receta: Receta): string {
  const d = receta.diseno ? deCompacto(compacto(receta.diseno as Diseno)) : null;
  // Solo va en corto si reconstruirlo da exactamente las mismas capas.
  if (d && JSON.stringify(construir(d, 'x').capas) === JSON.stringify(receta.capas)) {
    return PREFIJO_CORTO + a64(JSON.stringify(compacto(d)));
  }
  return PREFIJO + a64(JSON.stringify(receta));
}

/** El diseño que viene en un código (aunque llegue rodeado de texto), o null. */
export function decodificar(texto: string): Receta | null {
  const corto = texto.match(/sonidos2:([A-Za-z0-9_-]+)/);
  if (corto) {
    try {
      const d = deCompacto(JSON.parse(de64(corto[1])));
      if (!d) return null;
      const receta = construir(d);
      return receta.capas.length > 0 ? receta : null;
    } catch {
      return null;
    }
  }
  const m = texto.match(/sonidos1:([A-Za-z0-9_-]+)/);
  if (!m) return null;
  try {
    const r = JSON.parse(de64(m[1])) as Receta;
    if (r.version !== 1 || !Array.isArray(r.capas) || r.capas.length === 0) return null;
    const nombre = String(r.nombre?.es ?? 'Diseño').slice(0, 60);
    // Lo que llega de fuera entra siempre como diseño propio, con textos neutros.
    return {
      ...r,
      id: `propio.${Date.now().toString(36)}`,
      sala: 'propio',
      situacion: 'propio',
      nombre: { es: nombre, en: String(r.nombre?.en ?? nombre).slice(0, 60) },
      descripcion: { es: '', en: '' },
      ciencia: { es: 'Diseño personal: nadie lo ha evaluado.', en: 'Personal design: nobody has evaluated it.' },
      fuentes: [],
    };
  } catch {
    return null;
  }
}
