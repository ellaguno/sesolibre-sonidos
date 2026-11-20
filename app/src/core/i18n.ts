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
  'situacion.sonar': { es: 'Soñar en calma', en: 'Calm dreaming' },
  'situacion.despertar': { es: 'Despertar', en: 'Wake up' },
  'situacion.exploracion': { es: 'Frecuencias', en: 'Frequencies' },
  'situacion.viaje': { es: 'Viaje', en: 'Trip' },
  'sala.propio': { es: 'Tus diseños', en: 'Your designs' },
  'sala.propio.que': {
    es: 'Hechos por ti o compartidos contigo. Nadie los ha evaluado; el volumen seguro sí se respeta.',
    en: 'Made by you or shared with you. Nobody has evaluated them; the safe volume still applies.',
  },
  'situacion.propio': { es: 'Diseño propio', en: 'Own design' },
  disenar: { es: 'Diseñar', en: 'Design' },
  importar: { es: 'Importar', en: 'Import' },
  importarQue: { es: 'Pega aquí el código que te compartieron', en: 'Paste the code that was shared with you' },
  importarMal: { es: 'Ese código no es un diseño de Sonidos', en: 'That code is not a Sonidos design' },
  editar: { es: 'Editar', en: 'Edit' },
  compartir: { es: 'Compartir', en: 'Share' },
  copiado: { es: 'Código copiado', en: 'Code copied' },
  borrar: { es: 'Borrar', en: 'Delete' },
  guardar: { es: 'Guardar', en: 'Save' },
  nombre: { es: 'Nombre', en: 'Name' },
  miDiseno: { es: 'Mi diseño', en: 'My design' },
  'd.fondo': { es: 'Fondo (hasta tres)', en: 'Background (up to three)' },
  'd.colchon': { es: 'Colchón', en: 'Pad' },
  'd.modo': { es: 'Carácter', en: 'Mood' },
  'd.afinacion': { es: 'Afinación', en: 'Tuning' },
  'd.instrumento': { es: 'Instrumento', en: 'Instrument' },
  'd.densidad': { es: 'Notas', en: 'Notes' },
  'd.frecuencia': { es: 'Frecuencia', en: 'Frequency' },
  'd.hz': { es: 'Tono o portadora (Hz)', en: 'Tone or carrier (Hz)' },
  'd.beat': { es: 'Pulso o base (Hz)', en: 'Beat or base (Hz)' },
  'd.respiracion': { es: 'Guía de respiración', en: 'Breathing guide' },
  'd.vacio': { es: 'Elige al menos un sonido.', en: 'Pick at least one sound.' },
  'o.ninguno': { es: 'Nada', en: 'None' },
  'o.mar': { es: 'Mar submarino', en: 'Underwater sea' },
  'o.olas': { es: 'Olas en las rocas', en: 'Waves on rocks' },
  'o.burbujas': { es: 'Burbujas', en: 'Bubbles' },
  'o.oleaje': { es: 'Oleaje', en: 'Swell' },
  'o.viento': { es: 'Viento', en: 'Wind' },
  'o.lluvia': { es: 'Lluvia', en: 'Rain' },
  'o.aves': { es: 'Pajaritos sintéticos', en: 'Synthetic birds' },
  'o.rosa': { es: 'Ruido rosa', en: 'Pink noise' },
  'o.marron': { es: 'Ruido marrón', en: 'Brown noise' },
  'o.sintetico': { es: 'Sintético', en: 'Synthetic' },
  'o.cuerdas': { es: 'Cuerdas', en: 'Strings' },
  'o.cello': { es: 'Cellos', en: 'Cellos' },
  'o.violin': { es: 'Violines', en: 'Violins' },
  'o.flauta': { es: 'Flauta (viento)', en: 'Flute (wind)' },
  'o.corno': { es: 'Corno (metal)', en: 'Horn (brass)' },
  'd.evento': { es: 'De vez en cuando', en: 'Every so often' },
  'o.carillon': { es: 'Carillón de viento', en: 'Wind chimes' },
  'o.platillo': { es: 'Platillo frotado', en: 'Bowed cymbal' },
  'o.gong': { es: 'Gong', en: 'Gong' },
  'o.mirlo': { es: 'Mirlo', en: 'Blackbird' },
  'o.verderon': { es: 'Verderón', en: 'Greenfinch' },
  'o.mosquitero': { es: 'Mosquitero musical', en: 'Willow warbler' },
  'o.campanero': { es: 'Campanero (N. Zelanda)', en: 'Bellbird (NZ)' },
  'o.gaviotas': { es: 'Gaviotas', en: 'Gulls' },
  'o.selva': { es: 'Ave de selva (pita)', en: 'Jungle bird (pitta)' },
  'o.noche': { es: 'Noche (chotacabras)', en: 'Night (nightjar)' },
  'o.grillos': { es: 'Grillo topo', en: 'Mole cricket' },
  'o.ranas': { es: 'Ranitas', en: 'Tree frogs' },
  'o.trombones': { es: 'Trombones', en: 'Trombones' },
  'o.fagot': { es: 'Fagot', en: 'Bassoon' },
  'o.trompeta': { es: 'Trompeta', en: 'Trumpet' },
  'o.campanas': { es: 'Campanas tubulares', en: 'Tubular bells' },
  creditos: { es: 'Créditos de los sonidos', en: 'Sound credits' },
  'o.arpa': { es: 'Arpa', en: 'Harp' },
  'o.vibrafono': { es: 'Vibráfono', en: 'Vibraphone' },
  'o.marimba': { es: 'Marimba', en: 'Marimba' },
  'o.armonicos': { es: 'Armónicos de violín', en: 'Violin harmonics' },
  'o.re_pentatonico': { es: 'Luminoso', en: 'Bright' },
  'o.menor': { es: 'Hondo', en: 'Deep' },
  'o.quintas': { es: 'Neutro', en: 'Neutral' },
  'o.440': { es: '440 Hz', en: '440 Hz' },
  'o.432': { es: '432 Hz', en: '432 Hz' },
  'o.pocas': { es: 'Pocas', en: 'Few' },
  'o.algunas': { es: 'Algunas', en: 'Some' },
  'o.muchas': { es: 'Muchas', en: 'Many' },
  'o.tono': { es: 'Tono puro', en: 'Pure tone' },
  'o.binaural': { es: 'Binaural', en: 'Binaural' },
  'o.isocronico': { es: 'Isocrónico', en: 'Isochronic' },
  'o.serie': { es: 'Armónicos de una base', en: 'Harmonics of a base' },
  'o.si': { es: 'Sí', en: 'Yes' },
  'o.no': { es: 'No', en: 'No' },
  cienciaPropio: {
    es: 'Diseño personal: nadie lo ha evaluado. Los límites de volumen y de frecuencias son los mismos que en el resto de la app.',
    en: 'Personal design: nobody has evaluated it. Volume and frequency limits are the same as in the rest of the app.',
  },
  espectroPico: { es: 'Más fuerte', en: 'Strongest' },
  espectroToca: { es: 'Toca el gráfico para ver qué frecuencias hay en ese punto', en: 'Tap the graph to see the frequencies at that point' },
  espectroSimulado: { es: 'simulado', en: 'simulated' },
  espectro: { es: 'Espectro de lo que suena, de graves a agudos', en: 'Spectrum of what is playing, low to high' },
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
    es: 'Modo de demostración: el reloj corre, pero no hay sonido.',
    en: 'Demo mode: the clock runs, but there is no sound.',
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
