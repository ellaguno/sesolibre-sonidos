// Los diseños del usuario: unas pocas elecciones → una receta que el núcleo valida y toca
// como cualquier otra. Se guardan en el dispositivo y se comparten como un código de texto.

import type { Entrada, Receta } from './recetas';

export const FONDOS = ['mar', 'olas', 'burbujas', 'oleaje', 'viento', 'lluvia', 'aves', 'rosa', 'marron'] as const;
export const COLCHONES = ['ninguno', 'sintetico', 'cuerdas', 'cello', 'violin', 'flauta', 'corno'] as const;
export const MODOS = ['re_pentatonico', 'menor', 'quintas'] as const;
export const INSTRUMENTOS = ['ninguno', 'arpa', 'vibrafono', 'marimba', 'flauta', 'corno', 'armonicos'] as const;
export const FRECUENCIAS = ['ninguno', 'tono', 'binaural', 'isocronico', 'serie'] as const;
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
  minutos: number;
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
  const armonia = { modo: d.modo, afinacion_hz: d.afinacion, ciclo_s: 200 };
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
      octavas: d.instrumento === 'corno' ? [0, 1] : d.instrumento === 'flauta' || d.instrumento === 'armonicos' ? [2, 2] : [1, 2],
      nivel_db: ['flauta', 'armonicos', 'corno'].includes(d.instrumento) ? -29 : -23,
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
  if (d.respiracion) {
    capas.push({ tipo: 'respiracion', inhalar_s: 4, exhalar_s: 6, hz: 196, nivel_db: -17, transicion_s: 2 });
  }
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

const PREFIJO = 'sonidos1:';

/** Un diseño como texto que cabe en un mensaje: `sonidos1:` + JSON en base64url. */
export function codificar(receta: Receta): string {
  const bytes = new TextEncoder().encode(JSON.stringify(receta));
  let bin = '';
  bytes.forEach((b) => (bin += String.fromCharCode(b)));
  return PREFIJO + btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/** El diseño que viene en un código (aunque llegue rodeado de texto), o null. */
export function decodificar(texto: string): Receta | null {
  const m = texto.match(/sonidos1:([A-Za-z0-9_-]+)/);
  if (!m) return null;
  try {
    const bin = atob(m[1].replace(/-/g, '+').replace(/_/g, '/'));
    const r = JSON.parse(new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)))) as Receta;
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
