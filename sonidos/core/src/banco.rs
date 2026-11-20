//! El banco de muestras: grabaciones reales (instrumentos y ambientes) que algunas capas
//! tocan. El núcleo no abre archivos por su cuenta: la plataforma le pasa los bytes de los
//! que la receta necesita (`archivos_necesarios`) y aquí se decodifican.
//!
//! El índice (`assets/audio/banco.json`, generado por `tools/banco.py`) va embebido: da
//! nombre a cada instrumento y ambiente y dice qué archivo es cada nota.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, OnceLock};

use serde::Deserialize;

use crate::receta::{Generador, Receta};
use crate::{Error, SR};

/// PCM de 16 bits intercalado, siempre a 48 kHz.
pub struct Muestra {
    pub canales: usize,
    pub cuadros: usize,
    pub datos: Vec<i16>,
    /// RMS respecto a fondo de escala (0..1), para igualar niveles entre grabaciones.
    pub rms: f32,
}

impl Muestra {
    pub fn nueva(canales: usize, datos: Vec<i16>) -> Muestra {
        let canales = canales.max(1);
        let cuadros = datos.len() / canales;
        let suma: f64 = datos.iter().map(|&x| (x as f64 / 32768.0).powi(2)).sum();
        let rms = if datos.is_empty() { 0.0 } else { (suma / datos.len() as f64).sqrt() as f32 };
        Muestra { canales, cuadros, datos, rms }
    }

    #[inline]
    fn cuadro(&self, i: usize) -> (f32, f32) {
        const K: f32 = 1.0 / 32768.0;
        if self.canales == 1 {
            let x = self.datos[i] as f32 * K;
            (x, x)
        } else {
            let j = i * self.canales;
            (self.datos[j] as f32 * K, self.datos[j + 1] as f32 * K)
        }
    }

    /// Lee en una posición fraccionaria (interpolación lineal). Fuera de rango: silencio.
    #[inline]
    pub fn leer(&self, pos: f64) -> (f32, f32) {
        if pos < 0.0 {
            return (0.0, 0.0);
        }
        let i = pos as usize;
        if i + 1 >= self.cuadros {
            return if i < self.cuadros { self.cuadro(i) } else { (0.0, 0.0) };
        }
        let f = (pos - i as f64) as f32;
        let (al, ar) = self.cuadro(i);
        let (bl, br) = self.cuadro(i + 1);
        (al + (bl - al) * f, ar + (br - ar) * f)
    }

    pub fn segundos(&self) -> f32 {
        self.cuadros as f32 / SR as f32
    }
}

#[derive(Debug, Deserialize)]
pub struct InstrumentoDef {
    /// La nota se mantiene (cuerdas, flauta) o se pulsa y decae (arpa, vibráfono).
    pub sostenido: bool,
    /// Nota MIDI → archivo relativo a `assets/audio/`.
    pub notas: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Indice {
    pub version: u32,
    pub instrumentos: BTreeMap<String, InstrumentoDef>,
    pub ambientes: BTreeMap<String, String>,
    /// Ambientes que deben sonar a su velocidad real (cantos de animales).
    #[serde(default)]
    pub ambientes_fieles: Vec<String>,
    /// Ambientes que son música ya hecha: suenan en una sola copia.
    #[serde(default)]
    pub ambientes_unicos: Vec<String>,
    /// Golpes sueltos (gong, platillo, carillón): nombre → archivos entre los que se elige.
    #[serde(default)]
    pub eventos: BTreeMap<String, Vec<String>>,
}

pub fn indice() -> &'static Indice {
    static INDICE: OnceLock<Indice> = OnceLock::new();
    INDICE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/audio/banco.json")).expect("banco.json embebido inválido")
    })
}

pub fn instrumento(nombre: &str) -> Result<&'static InstrumentoDef, Error> {
    indice().instrumentos.get(nombre).ok_or_else(|| {
        let hay: Vec<&str> = indice().instrumentos.keys().map(String::as_str).collect();
        Error::Receta(format!("instrumento desconocido '{nombre}' (hay: {})", hay.join(", ")))
    })
}

pub fn ambiente(nombre: &str) -> Result<&'static str, Error> {
    indice().ambientes.get(nombre).map(String::as_str).ok_or_else(|| {
        let hay: Vec<&str> = indice().ambientes.keys().map(String::as_str).collect();
        Error::Receta(format!("ambiente desconocido '{nombre}' (hay: {})", hay.join(", ")))
    })
}

pub fn evento(nombre: &str) -> Result<&'static [String], Error> {
    indice().eventos.get(nombre).map(Vec::as_slice).ok_or_else(|| {
        let hay: Vec<&str> = indice().eventos.keys().map(String::as_str).collect();
        Error::Receta(format!("evento desconocido '{nombre}' (hay: {})", hay.join(", ")))
    })
}

/// Los archivos (relativos a `assets/audio/`) que la plataforma debe cargar para esta receta.
pub fn archivos_necesarios(receta: &Receta) -> Result<Vec<String>, Error> {
    let mut v: Vec<String> = Vec::new();
    for c in &receta.capas {
        match &c.gen {
            Generador::Ambiente { ambiente: a, .. } => v.push(ambiente(a)?.to_string()),
            Generador::Instrumento { instrumento: i, .. } => v.extend(instrumento(i)?.notas.values().cloned()),
            Generador::Eventos { evento: e, .. } => v.extend(evento(e)?.iter().cloned()),
            Generador::Colchon { timbre: Some(t), .. } => v.extend(instrumento(t)?.notas.values().cloned()),
            _ => {}
        }
    }
    v.sort();
    v.dedup();
    Ok(v)
}

/// Frecuencia de una nota MIDI con La4 = 440 Hz (así están grabadas las muestras).
pub fn midi_a_hz(nota: i32) -> f32 {
    440.0 * 2f32.powf((nota - 69) as f32 / 12.0)
}

/// Las notas de un instrumento, ya resueltas contra el banco: (Hz de la grabación, muestra).
pub fn notas_de(nombre: &str, banco: &Banco) -> Result<Vec<(f32, Arc<Muestra>)>, Error> {
    let def = instrumento(nombre)?;
    let mut v = Vec::new();
    for (nota, archivo) in &def.notas {
        let n: i32 = nota.parse().map_err(|_| Error::Receta(format!("nota inválida '{nota}' en banco.json")))?;
        v.push((midi_a_hz(n), banco.obtener(archivo)?));
    }
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    if v.is_empty() {
        return Err(Error::Receta(format!("el instrumento '{nombre}' no tiene notas")));
    }
    Ok(v)
}

/// La grabación más cercana en altura a `hz`.
pub fn mas_cercana(notas: &[(f32, Arc<Muestra>)], hz: f32) -> &(f32, Arc<Muestra>) {
    notas
        .iter()
        .min_by(|a, b| (a.0 / hz).ln().abs().total_cmp(&(b.0 / hz).ln().abs()))
        .expect("instrumento sin notas")
}

#[derive(Default)]
pub struct Banco {
    muestras: HashMap<String, Arc<Muestra>>,
}

impl Banco {
    pub fn vacio() -> Banco {
        Banco::default()
    }

    pub fn agregar_pcm(&mut self, nombre: &str, canales: usize, datos: Vec<i16>) {
        self.muestras.insert(nombre.to_string(), Arc::new(Muestra::nueva(canales, datos)));
    }

    /// Decodifica un Ogg Vorbis (48 kHz, mono o estéreo) y lo guarda con ese nombre.
    #[cfg(feature = "ogg")]
    pub fn agregar_ogg(&mut self, nombre: &str, bytes: &[u8]) -> Result<(), Error> {
        let mal = |e: String| Error::Receta(format!("{nombre}: {e}"));
        let mut r = lewton::inside_ogg::OggStreamReader::new(std::io::Cursor::new(bytes)).map_err(|e| mal(e.to_string()))?;
        let canales = r.ident_hdr.audio_channels as usize;
        if r.ident_hdr.audio_sample_rate != SR {
            return Err(mal(format!("{} Hz; el banco es de {} Hz", r.ident_hdr.audio_sample_rate, SR)));
        }
        if canales == 0 || canales > 2 {
            return Err(mal(format!("{canales} canales")));
        }
        let mut datos: Vec<i16> = Vec::new();
        while let Some(p) = r.read_dec_packet_itl().map_err(|e| mal(e.to_string()))? {
            datos.extend_from_slice(&p);
        }
        self.agregar_pcm(nombre, canales, datos);
        Ok(())
    }

    /// Carga del disco los archivos que la receta necesita. Para el CLI y las pruebas.
    #[cfg(feature = "ogg")]
    pub fn para_receta(receta: &Receta, raiz: &std::path::Path) -> Result<Banco, Error> {
        let mut b = Banco::vacio();
        for a in archivos_necesarios(receta)? {
            let bytes = std::fs::read(raiz.join(&a)).map_err(|e| Error::Receta(format!("{}: {e}", raiz.join(&a).display())))?;
            b.agregar_ogg(&a, &bytes)?;
        }
        Ok(b)
    }

    pub fn obtener(&self, nombre: &str) -> Result<Arc<Muestra>, Error> {
        self.muestras.get(nombre).cloned().ok_or_else(|| Error::Receta(format!("falta la muestra '{nombre}' en el banco")))
    }

    pub fn len(&self) -> usize {
        self.muestras.len()
    }

    pub fn is_empty(&self) -> bool {
        self.muestras.is_empty()
    }
}
