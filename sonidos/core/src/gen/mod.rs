//! Fuentes de sonido. Cada una escribe cuadros estéreo a nivel nominal; la mezcla pone el
//! nivel, la ventana y el techo.

mod barrido;
mod colchon;
mod pulso;
mod respiracion;
mod ruido;
mod tono;

use crate::receta::Generador;
use crate::rng::Rng;

/// Un cuadro estéreo.
pub type Cuadro = [f32; 2];

pub trait Fuente {
    /// Escribe `out.len()` cuadros. `t0` es el tiempo del primer cuadro en segundos.
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]);
}

/// Construye la fuente de un generador. `duracion_s` sirve a las derivas (p. ej. un beat que
/// baja a lo largo de la receta).
pub fn construir(gen: &Generador, sr: f32, rng: Rng, duracion_s: f32) -> Option<Box<dyn Fuente>> {
    Some(match gen {
        Generador::Ruido { color, paso_bajo_hz, paso_alto_hz } => {
            Box::new(ruido::Ruido::nuevo(*color, *paso_bajo_hz, *paso_alto_hz, sr, rng))
        }
        Generador::Tono { hz, armonicos } => Box::new(tono::Tono::nuevo(*hz, *armonicos, sr)),
        Generador::Pulso { forma, portadora_hz, beat_hz, beat_hz_final } => Box::new(pulso::Pulso::nuevo(
            *forma,
            *portadora_hz,
            *beat_hz,
            beat_hz_final.unwrap_or(*beat_hz),
            duracion_s,
            sr,
        )),
        Generador::Colchon { afinacion_hz, raiz_hz, modo, ciclo_s, voces, brillo } => Box::new(colchon::Colchon::nuevo(
            raiz_hz.unwrap_or_else(|| crate::teoria::raiz_por_defecto(*afinacion_hz)),
            modo,
            *ciclo_s,
            *voces as usize,
            brillo.unwrap_or(0.4),
            sr,
            rng,
        )),
        Generador::Respiracion { inhalar_s, retener_s, exhalar_s, pausa_s, hz } => Box::new(
            respiracion::Respiracion::nueva(*inhalar_s, *retener_s, *exhalar_s, *pausa_s, *hz, sr, rng),
        ),
        // Fase 2: bucles de campo. Hasta entonces la capa se ignora en silencio.
        Generador::Ambiente { .. } => return None,
        Generador::Barrido { desde_hz, hasta_hz, periodo_s } => {
            Box::new(barrido::Barrido::nuevo(*desde_hz, *hasta_hz, *periodo_s, sr))
        }
    })
}
