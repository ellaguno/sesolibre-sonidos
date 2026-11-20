//! Fuentes de sonido. Cada una escribe cuadros estéreo a nivel nominal; la mezcla pone el
//! nivel, la ventana y el techo.

mod ambiente;
mod barrido;
mod colchon;
mod instrumento;
mod naturaleza;
mod pulso;
mod respiracion;
mod ruido;
mod serie;
mod tono;

use crate::banco::{self, Banco};
use crate::receta::Generador;
use crate::rng::Rng;
use crate::Error;

/// Un cuadro estéreo.
pub type Cuadro = [f32; 2];

pub trait Fuente: Send {
    /// Escribe `out.len()` cuadros. `t0` es el tiempo del primer cuadro en segundos.
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]);
}

/// Construye la fuente de un generador. `duracion_s` sirve a las derivas (p. ej. un beat que
/// baja a lo largo de la receta). Las capas con muestras las toman del banco.
pub fn construir(gen: &Generador, sr: f32, rng: Rng, duracion_s: f32, banco: &Banco) -> Result<Box<dyn Fuente>, Error> {
    Ok(match gen {
        Generador::Ruido { color, paso_bajo_hz, paso_alto_hz, oleaje_s } => {
            Box::new(ruido::Ruido::nuevo(*color, *paso_bajo_hz, *paso_alto_hz, *oleaje_s, sr, rng))
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
        Generador::Colchon { afinacion_hz, raiz_hz, modo, ciclo_s, voces, brillo, timbre } => {
            let notas = match timbre {
                Some(t) => Some(banco::notas_de(t, banco)?),
                None => None,
            };
            Box::new(colchon::Colchon::nuevo(
                raiz_hz.unwrap_or_else(|| crate::teoria::raiz_por_defecto(*afinacion_hz)),
                modo,
                *ciclo_s,
                *voces as usize,
                brillo.unwrap_or(0.4),
                notas,
                sr,
                rng,
            ))
        }
        Generador::Instrumento { instrumento, afinacion_hz, raiz_hz, modo, ciclo_s, notas_por_minuto, octavas } => {
            Box::new(instrumento::Instrumento::nuevo(
                banco::notas_de(instrumento, banco)?,
                banco::instrumento(instrumento)?.sostenido,
                raiz_hz.unwrap_or_else(|| crate::teoria::raiz_por_defecto(*afinacion_hz)),
                modo,
                *ciclo_s,
                *notas_por_minuto,
                octavas.map(|o| (o[0], o[1])).unwrap_or((1, 2)),
                sr,
                rng,
            ))
        }
        Generador::Respiracion { inhalar_s, retener_s, exhalar_s, pausa_s, hz } => Box::new(
            respiracion::Respiracion::nueva(*inhalar_s, *retener_s, *exhalar_s, *pausa_s, *hz, sr, rng),
        ),
        Generador::Ambiente { ambiente, paso_bajo_hz } => {
            let fiel = banco::indice().ambientes_fieles.iter().any(|a| a == ambiente);
            Box::new(ambiente::Ambiente::nuevo(banco.obtener(banco::ambiente(ambiente)?)?, *paso_bajo_hz, fiel, sr))
        }
        Generador::Serie { base_hz, armonicos } => Box::new(serie::Serie::nueva(*base_hz, armonicos, sr)),
        Generador::Naturaleza { clase } => naturaleza::nueva(*clase, sr, rng),
        Generador::Barrido { desde_hz, hasta_hz, periodo_s } => {
            Box::new(barrido::Barrido::nuevo(*desde_hz, *hasta_hz, *periodo_s, sr))
        }
    })
}
