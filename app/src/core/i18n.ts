import type { Idioma, Texto } from './recetas';

const DIC = {
  lema: { es: 'Sonido para una situación', en: 'Sound for a situation' },
  noche: { es: 'Noche', en: 'Night' },
  nocheAyuda: { es: 'Modo noche: todo suena 8 dB más bajo.', en: 'Night mode: everything plays 8 dB lower.' },
  'sala.comprobado': { es: 'Comprobado', en: 'Proven' },
  'sala.prometedor': { es: 'Prometedor', en: 'Promising' },
  'sala.exploracion': { es: 'Exploración', en: 'Exploration' },
  'sala.comprobado.que': {
    es: 'Evidencia moderada o buena. Uso seguro.',
    en: 'Moderate or good evidence. Safe to use.',
  },
  'sala.prometedor.que': {
    es: 'Evidencia mixta o preliminar. Uso seguro.',
    en: 'Mixed or preliminary evidence. Safe to use.',
  },
  'sala.exploracion.que': {
    es: 'Sin evidencia científica. Exploración personal: prueba y saca tus conclusiones.',
    en: 'No scientific evidence. Personal exploration: try it and draw your own conclusions.',
  },
  'situacion.dormir': { es: 'Dormir', en: 'Sleep' },
  'situacion.concentrar': { es: 'Concentrar', en: 'Focus' },
  'situacion.respirar': { es: 'Respirar', en: 'Breathe' },
  'situacion.meditar': { es: 'Meditar', en: 'Meditate' },
  'situacion.calmar': { es: 'Calmar', en: 'Calm' },
  'situacion.exploracion': { es: 'Frecuencias', en: 'Frequencies' },
  'situacion.viaje': { es: 'Viaje', en: 'Trip' },
  ciencia: { es: 'Qué dice la ciencia', en: 'What science says' },
  fuentes: { es: 'Fuentes', en: 'Sources' },
  duracion: { es: 'Duración', en: 'Duration' },
  tocar: { es: 'Tocar', en: 'Play' },
  cerrar: { es: 'Cerrar', en: 'Close' },
  pausar: { es: 'Pausar', en: 'Pause' },
  reanudar: { es: 'Reanudar', en: 'Resume' },
  parar: { es: 'Parar', en: 'Stop' },
  mas15: { es: '+15 min', en: '+15 min' },
  quedan: { es: 'quedan', en: 'left' },
  cargando: { es: 'Preparando…', en: 'Getting ready…' },
  enPausa: { es: 'En pausa', en: 'Paused' },
  apagada: {
    es: 'Puedes apagar la pantalla: sigue sonando y se apaga solo.',
    en: 'You can turn the screen off: it keeps playing and stops by itself.',
  },
  simulado: {
    es: 'Vista previa en el navegador: el reloj corre, pero aquí todavía no suena. El sonido está en la app de Android.',
    en: 'Browser preview: the clock runs, but there is no sound here yet. Sound lives in the Android app.',
  },
  sinAuriculares: {
    es: 'Esta receta necesita auriculares y no veo ninguno conectado. Por el altavoz el pulso binaural no funciona.',
    en: 'This recipe needs headphones and none seem connected. Through the speaker the binaural pulse does not work.',
  },
  'adv.auriculares': { es: 'Necesita auriculares (un tono por oído).', en: 'Needs headphones (one tone per ear).' },
  'adv.no_conducir': {
    es: 'No lo uses al conducir ni al operar máquinas.',
    en: 'Do not use while driving or operating machinery.',
  },
  'adv.epilepsia': {
    es: 'Si tienes epilepsia o fotosensibilidad, consulta antes.',
    en: 'If you have epilepsy or photosensitivity, ask first.',
  },
  'adv.parar_si_molesta': {
    es: 'Si molesta o da dolor de cabeza, para.',
    en: 'If it bothers you or gives you a headache, stop.',
  },
  'adv.siempre': { es: 'No sustituye atención médica.', en: 'Not a substitute for medical care.' },
  pie: {
    es: 'El volumen lo pones tú: empieza bajo. De noche, con auriculares, más bajo aún.',
    en: 'You set the volume: start low. At night, with headphones, lower still.',
  },
  min: { es: 'min', en: 'min' },
  h: { es: 'h', en: 'h' },
} as const;

export type Clave = keyof typeof DIC;

export function idiomaInicial(): Idioma {
  try {
    const guardado = localStorage.getItem('sonidos.idioma');
    if (guardado === 'es' || guardado === 'en') return guardado;
  } catch {
    /* sin almacenamiento: se usa el del sistema */
  }
  return typeof navigator !== 'undefined' && navigator.language.toLowerCase().startsWith('en') ? 'en' : 'es';
}

export function traducir(idioma: Idioma, clave: Clave | string): string {
  const e = (DIC as Record<string, Texto>)[clave];
  return e ? e[idioma] : clave;
}

export function minutosLegibles(idioma: Idioma, minutos: number): string {
  if (minutos >= 120 && minutos % 60 === 0) return `${minutos / 60} ${traducir(idioma, 'h')}`;
  return `${minutos} ${traducir(idioma, 'min')}`;
}

export function reloj(segundos: number): string {
  const s = Math.max(0, Math.round(segundos));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const ss = String(s % 60).padStart(2, '0');
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${ss}` : `${m}:${ss}`;
}
