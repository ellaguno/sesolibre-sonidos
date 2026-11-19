//! El formato de receta: el contrato entre el núcleo y cualquier interfaz
//! (la app, Somnia, Mind). Versionado. Ver DISEÑO.md §4.

use serde::{Deserialize, Serialize};

use crate::Error;

pub const VERSION_RECETA: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Texto {
    pub es: String,
    pub en: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sala {
    Comprobado,
    Prometedor,
    Exploracion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receta {
    pub version: u32,
    pub id: String,
    pub sala: Sala,
    pub situacion: String,
    pub nombre: Texto,
    #[serde(default)]
    pub descripcion: Texto,
    /// La línea "qué dice la ciencia". Obligatoria de hecho: `validar` la exige.
    #[serde(default)]
    pub ciencia: Texto,
    #[serde(default)]
    pub fuentes: Vec<String>,
    pub duracion_s: f32,
    #[serde(default = "d_fade_in")]
    pub fade_in_s: f32,
    #[serde(default = "d_fade_out")]
    pub fade_out_s: f32,
    #[serde(default)]
    pub semilla: u64,
    pub capas: Vec<Capa>,
    #[serde(default = "d_techo")]
    pub techo_db: f32,
    #[serde(default)]
    pub advertencias: Vec<String>,
    /// La sala donde suenan las capas que envían a la reverberación (`reverb` > 0).
    #[serde(default)]
    pub espacio: Option<Espacio>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Espacio {
    /// Segundos que tarda la cola en apagarse (RT60).
    #[serde(default = "d_rt60")]
    pub rt60_s: f32,
    #[serde(default = "d_amortiguacion")]
    pub amortiguacion_hz: f32,
}

impl Default for Espacio {
    fn default() -> Self {
        Espacio { rt60_s: d_rt60(), amortiguacion_hz: d_amortiguacion() }
    }
}

fn d_fade_in() -> f32 {
    8.0
}
fn d_fade_out() -> f32 {
    60.0
}
fn d_techo() -> f32 {
    -14.0
}
fn d_transicion() -> f32 {
    10.0
}
fn d_afinacion() -> f32 {
    440.0
}
fn d_modo() -> String {
    "re_pentatonico".into()
}
fn d_ciclo() -> f32 {
    240.0
}
fn d_voces() -> u8 {
    5
}
fn d_hz_resp() -> f32 {
    196.0
}
fn d_rt60() -> f32 {
    5.0
}
fn d_amortiguacion() -> f32 {
    3500.0
}
fn d_npm() -> f32 {
    6.0
}

/// Una capa: un generador con su nivel y su ventana de actividad dentro de la receta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capa {
    /// 0 dB = nivel nominal del generador (tonos: pico ≈ 1; ruido: RMS ≈ 0,25).
    pub nivel_db: f32,
    #[serde(default)]
    pub desde_s: f32,
    #[serde(default)]
    pub hasta_s: Option<f32>,
    /// Segundos de entrada/salida de la capa dentro de su ventana.
    #[serde(default = "d_transicion")]
    pub transicion_s: f32,
    /// Cuánto de la capa va a la reverberación (0 = seca, 1 = tanto como la señal directa).
    #[serde(default)]
    pub reverb: f32,
    #[serde(flatten)]
    pub gen: Generador,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ColorRuido {
    Blanco,
    Rosa,
    Marron,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormaPulso {
    /// Un tono por oído; la diferencia es el *beat*. Requiere auriculares.
    Binaural,
    /// Un tono encendido y apagado suavemente al ritmo del *beat*. Sirve en altavoz.
    Isocronico,
    /// Modulación de amplitud (p. ej. 40 Hz) sobre una portadora.
    Am,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Generador {
    Ruido {
        color: ColorRuido,
        #[serde(default)]
        paso_bajo_hz: Option<f32>,
        #[serde(default)]
        paso_alto_hz: Option<f32>,
        /// Periodo en segundos de un vaivén de mar: el ruido sube y baja como olas.
        #[serde(default)]
        oleaje_s: Option<f32>,
    },
    Tono {
        hz: f32,
        /// Parciales extra (2.º, 3.º…) a -12 dB cada uno. 0 = seno puro.
        #[serde(default)]
        armonicos: u8,
    },
    Pulso {
        forma: FormaPulso,
        portadora_hz: f32,
        beat_hz: f32,
        /// Si se da, el *beat* deriva linealmente hasta aquí al final de la receta.
        #[serde(default)]
        beat_hz_final: Option<f32>,
    },
    Colchon {
        #[serde(default = "d_afinacion")]
        afinacion_hz: f32,
        /// Raíz explícita (p. ej. 264 Hz para un colchón "528"). Si falta: Re2 bajo la afinación.
        #[serde(default)]
        raiz_hz: Option<f32>,
        #[serde(default = "d_modo")]
        modo: String,
        #[serde(default = "d_ciclo")]
        ciclo_s: f32,
        #[serde(default = "d_voces")]
        voces: u8,
        /// 0 = solo fundamentales; 1 = parciales presentes. Por defecto 0,4.
        #[serde(default)]
        brillo: Option<f32>,
        /// Instrumento sostenido del banco (p. ej. "cuerdas"). Si falta: sintético.
        #[serde(default)]
        timbre: Option<String>,
    },
    /// Un instrumento real (arpa, vibráfono, flauta…) que toca notas sueltas del acorde que
    /// suena en el colchón. Para ir juntos basta con el mismo modo, raíz y ciclo.
    Instrumento {
        instrumento: String,
        #[serde(default = "d_afinacion")]
        afinacion_hz: f32,
        #[serde(default)]
        raiz_hz: Option<f32>,
        #[serde(default = "d_modo")]
        modo: String,
        #[serde(default = "d_ciclo")]
        ciclo_s: f32,
        #[serde(default = "d_npm")]
        notas_por_minuto: f32,
        /// Octavas por encima del colchón entre las que elige: [mínima, máxima].
        #[serde(default)]
        octavas: Option<[i32; 2]>,
    },
    Respiracion {
        inhalar_s: f32,
        #[serde(default)]
        retener_s: f32,
        exhalar_s: f32,
        #[serde(default)]
        pausa_s: f32,
        #[serde(default = "d_hz_resp")]
        hz: f32,
    },
    /// Grabación de campo en bucle (agua, olas…), por su nombre en el banco.
    Ambiente {
        ambiente: String,
        #[serde(default)]
        paso_bajo_hz: Option<f32>,
    },
    /// Solo Exploración: frecuencia que recorre un rango ida y vuelta (cimática).
    Barrido {
        desde_hz: f32,
        hasta_hz: f32,
        periodo_s: f32,
    },
}

/// Palabras que una receta no puede usar en sus textos (DISEÑO.md §8). Se compara por
/// palabra completa, sin mayúsculas ni acentos.
pub const PALABRAS_PROHIBIDAS: &[&str] = &[
    "cura", "curar", "curativo", "curativa", "sana", "sanar", "sanacion", "sanador", "sanadora",
    "trata", "tratar", "tratamiento", "terapia", "terapeutico", "terapeutica", "medicina",
    "cure", "cures", "heal", "heals", "healing", "treat", "treats", "treatment", "therapy",
    "therapeutic", "medicine",
];

impl Receta {
    pub fn desde_json(s: &str) -> Result<Receta, Error> {
        let r: Receta = serde_json::from_str(s)?;
        r.validar()?;
        Ok(r)
    }

    /// La misma receta con otra duración (el temporizador del usuario). Las capas con
    /// final propio se recortan; el desvanecimiento final nunca pasa de un tercio.
    pub fn con_duracion(&self, duracion_s: f32) -> Receta {
        let mut r = self.clone();
        r.duracion_s = duracion_s.clamp(10.0, 12.0 * 3600.0);
        r.fade_out_s = r.fade_out_s.min(r.duracion_s / 3.0).max(2.0);
        r.fade_in_s = r.fade_in_s.min(r.duracion_s / 3.0).max(2.0);
        for c in r.capas.iter_mut() {
            if let Some(h) = c.hasta_s {
                c.hasta_s = if h >= self.duracion_s { None } else { Some(h.min(r.duracion_s)) };
            }
        }
        r.capas.retain(|c| c.desde_s < r.duracion_s && c.hasta_s.map(|h| h > c.desde_s).unwrap_or(true));
        r
    }

    pub fn a_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Comprueba los límites de seguridad y de honestidad. Una receta que no pasa no suena.
    pub fn validar(&self) -> Result<(), Error> {
        let e = |m: String| Err(Error::Receta(format!("{}: {}", self.id, m)));
        if self.version != VERSION_RECETA {
            return e(format!("versión {} (se esperaba {})", self.version, VERSION_RECETA));
        }
        if !(10.0..=12.0 * 3600.0).contains(&self.duracion_s) {
            return e("duración fuera de [10 s, 12 h]".into());
        }
        if self.techo_db > -6.0 {
            return e("el techo no puede superar -6 dBFS".into());
        }
        if self.fade_in_s < 2.0 || self.fade_out_s < 2.0 {
            return e("los desvanecimientos duran al menos 2 s (nada de golpes)".into());
        }
        if self.ciencia.es.trim().is_empty() || self.ciencia.en.trim().is_empty() {
            return e("falta la línea 'ciencia' (es y en): toda receta dice qué se sabe".into());
        }
        if self.capas.is_empty() {
            return e("sin capas".into());
        }
        if let Some(esp) = &self.espacio {
            if !(0.3..=12.0).contains(&esp.rt60_s) || !(500.0..=12_000.0).contains(&esp.amortiguacion_hz) {
                return e("espacio: rt60 en [0.3, 12] s y amortiguación en [500, 12000] Hz".into());
            }
        }
        let mut hay_binaural = false;
        let mut hay_pulso = false;
        for (i, c) in self.capas.iter().enumerate() {
            let ec = |m: &str| Err(Error::Receta(format!("{} capa {}: {}", self.id, i, m)));
            if c.nivel_db > 0.0 {
                return ec("el nivel de una capa no supera 0 dB");
            }
            if c.transicion_s < 1.0 {
                return ec("la transición dura al menos 1 s");
            }
            if !(0.0..=1.0).contains(&c.reverb) {
                return ec("el envío a reverb va de 0 a 1");
            }
            if let Some(h) = c.hasta_s {
                if h <= c.desde_s {
                    return ec("hasta_s debe ser mayor que desde_s");
                }
            }
            match &c.gen {
                Generador::Ruido { paso_bajo_hz, paso_alto_hz, oleaje_s, .. } => {
                    for f in [paso_bajo_hz, paso_alto_hz].into_iter().flatten() {
                        if !(20.0..=16_000.0).contains(f) {
                            return ec("filtro fuera de [20, 16000] Hz");
                        }
                    }
                    if let Some(o) = oleaje_s {
                        if !(4.0..=60.0).contains(o) {
                            return ec("oleaje fuera de [4, 60] s");
                        }
                    }
                }
                Generador::Tono { hz, armonicos } => {
                    if !(20.0..=16_000.0).contains(hz) {
                        return ec("tono fuera de [20, 16000] Hz: ni infrasonido ni ultrasonido");
                    }
                    if *armonicos > 6 {
                        return ec("como mucho 6 armónicos");
                    }
                }
                Generador::Pulso { forma, portadora_hz, beat_hz, beat_hz_final } => {
                    if !(60.0..=1_000.0).contains(portadora_hz) {
                        return ec("portadora fuera de [60, 1000] Hz");
                    }
                    for b in std::iter::once(beat_hz).chain(beat_hz_final.iter()) {
                        if !(0.5..=45.0).contains(b) {
                            return ec("beat fuera de [0.5, 45] Hz");
                        }
                    }
                    hay_pulso = true;
                    if *forma == FormaPulso::Binaural {
                        hay_binaural = true;
                    }
                }
                Generador::Colchon { afinacion_hz, raiz_hz, ciclo_s, voces, timbre, .. } => {
                    if let Some(t) = timbre {
                        if !crate::banco::instrumento(t).map_err(|e| Error::Receta(format!("{} capa {}: {}", self.id, i, e)))?.sostenido {
                            return ec("el timbre del colchón debe ser un instrumento sostenido");
                        }
                    }
                    if !(400.0..=480.0).contains(afinacion_hz) {
                        return ec("afinación fuera de [400, 480] Hz");
                    }
                    if let Some(r) = raiz_hz {
                        if !(40.0..=400.0).contains(r) {
                            return ec("raíz fuera de [40, 400] Hz");
                        }
                    }
                    if *ciclo_s < 20.0 || *voces == 0 || *voces > 8 {
                        return ec("ciclo ≥ 20 s y voces en [1, 8]");
                    }
                }
                Generador::Respiracion { inhalar_s, retener_s, exhalar_s, pausa_s, hz } => {
                    if *inhalar_s < 1.0 || *exhalar_s < 1.0 || *retener_s < 0.0 || *pausa_s < 0.0 {
                        return ec("tiempos de respiración inválidos");
                    }
                    if !(60.0..=600.0).contains(hz) {
                        return ec("tono de respiración fuera de [60, 600] Hz");
                    }
                }
                Generador::Instrumento { instrumento, afinacion_hz, raiz_hz, ciclo_s, notas_por_minuto, octavas, .. } => {
                    crate::banco::instrumento(instrumento).map_err(|e| Error::Receta(format!("{} capa {}: {}", self.id, i, e)))?;
                    if !(400.0..=480.0).contains(afinacion_hz) {
                        return ec("afinación fuera de [400, 480] Hz");
                    }
                    if let Some(r) = raiz_hz {
                        if !(40.0..=400.0).contains(r) {
                            return ec("raíz fuera de [40, 400] Hz");
                        }
                    }
                    if *ciclo_s < 20.0 || !(0.5..=30.0).contains(notas_por_minuto) {
                        return ec("ciclo ≥ 20 s y notas por minuto en [0.5, 30]");
                    }
                    if let Some([a, b]) = octavas {
                        if a > b || *a < -2 || *b > 5 {
                            return ec("octavas: [mínima, máxima] dentro de [-2, 5]");
                        }
                    }
                }
                Generador::Ambiente { ambiente, paso_bajo_hz } => {
                    crate::banco::ambiente(ambiente).map_err(|e| Error::Receta(format!("{} capa {}: {}", self.id, i, e)))?;
                    if let Some(f) = paso_bajo_hz {
                        if !(20.0..=16_000.0).contains(f) {
                            return ec("filtro fuera de [20, 16000] Hz");
                        }
                    }
                }
                Generador::Barrido { desde_hz, hasta_hz, periodo_s } => {
                    if self.sala != Sala::Exploracion {
                        return ec("el barrido solo existe en Exploración");
                    }
                    if !(20.0..=16_000.0).contains(desde_hz) || !(20.0..=16_000.0).contains(hasta_hz) {
                        return ec("barrido fuera de [20, 16000] Hz");
                    }
                    if *periodo_s < 2.0 {
                        return ec("periodo de barrido ≥ 2 s");
                    }
                }
            }
        }
        if hay_binaural && !self.advertencias.iter().any(|a| a == "auriculares") {
            return e("una receta binaural lleva la advertencia 'auriculares'".into());
        }
        if hay_pulso && !self.advertencias.iter().any(|a| a == "no_conducir") {
            return e("una receta con pulsos lleva la advertencia 'no_conducir'".into());
        }
        if let Some(p) = self.palabra_prohibida() {
            return e(format!("usa la palabra prohibida '{}' (DISEÑO.md §8)", p));
        }
        Ok(())
    }

    fn palabra_prohibida(&self) -> Option<&'static str> {
        let textos = [&self.nombre, &self.descripcion, &self.ciencia];
        for t in textos {
            for s in [&t.es, &t.en] {
                for palabra in s.split(|c: char| !c.is_alphanumeric()) {
                    let p = sin_acentos(palabra);
                    if let Some(x) = PALABRAS_PROHIBIDAS.iter().find(|&&x| x == p) {
                        return Some(x);
                    }
                }
            }
        }
        None
    }
}

fn sin_acentos(s: &str) -> String {
    s.chars()
        .map(|c| match c.to_lowercase().next().unwrap_or(c) {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}
