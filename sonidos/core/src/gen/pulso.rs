use std::f32::consts::PI;

use super::{Cuadro, Fuente};
use crate::dsp::Seno;
use crate::receta::FormaPulso;

/// Pulsos binaurales, isocrónicos o AM. El beat deriva linealmente de `beat_a` a `beat_b`
/// a lo largo de la receta (p. ej. de 10 Hz a 2 Hz para acompañar el sueño).
pub struct Pulso {
    forma: FormaPulso,
    portadora: f32,
    beat_a: f32,
    beat_b: f32,
    duracion: f32,
    izq: Seno,
    der: Seno,
    sr: f32,
}

impl Pulso {
    pub fn nuevo(forma: FormaPulso, portadora: f32, beat_a: f32, beat_b: f32, duracion: f32, sr: f32) -> Self {
        Pulso { forma, portadora, beat_a, beat_b, duracion: duracion.max(1.0), izq: Seno::nuevo(sr, 0.0), der: Seno::nuevo(sr, 0.0), sr }
    }
}

impl Fuente for Pulso {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        for (i, c) in out.iter_mut().enumerate() {
            let t = t0 + i as f64 * dt;
            let x = (t as f32 / self.duracion).clamp(0.0, 1.0);
            let beat = self.beat_a + (self.beat_b - self.beat_a) * x;
            match self.forma {
                FormaPulso::Binaural => {
                    // Portadora al centro, cada oído a ±beat/2: el tono percibido es la portadora.
                    let l = self.izq.siguiente(self.portadora - beat / 2.0);
                    let r = self.der.siguiente(self.portadora + beat / 2.0);
                    *c = [l, r];
                }
                FormaPulso::Isocronico => {
                    let tono = self.izq.siguiente(self.portadora);
                    // Ventana de coseno alzado: encendido/apagado sin bordes.
                    let fase = (t as f32 * beat).fract();
                    let v = 0.5 - 0.5 * (2.0 * PI * fase).cos();
                    let y = tono * v;
                    *c = [y, y];
                }
                FormaPulso::Am => {
                    let tono = self.izq.siguiente(self.portadora);
                    let fase = (t as f32 * beat).fract();
                    let m = 0.6 + 0.4 * (2.0 * PI * fase).cos();
                    let y = tono * m;
                    *c = [y, y];
                }
            }
        }
    }
}
