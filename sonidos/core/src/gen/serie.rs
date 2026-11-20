use super::{Cuadro, Fuente};
use crate::dsp::Seno;

/// Armónicos audibles de una fundamental que no se oye (p. ej. 7,83 Hz, la resonancia
/// Schumann). Todos son múltiplos exactos, así que la onda entera se repite a la
/// frecuencia base: el oído percibe ese pulso aunque la base no suene. Nominal: pico ≈ 1.
pub struct Serie {
    base_hz: f32,
    parciales: Vec<(f32, f32, Seno)>, // (múltiplo, amplitud, oscilador)
    norma: f32,
}

impl Serie {
    pub fn nueva(base_hz: f32, armonicos: &[u16], sr: f32) -> Self {
        let n0 = armonicos.iter().copied().min().unwrap_or(1).max(1) as f32;
        // Los más graves mandan; los altos se apagan como en un sonido natural.
        let parciales: Vec<(f32, f32, Seno)> =
            armonicos.iter().map(|&n| (n as f32, (n0 / n as f32).powf(0.8), Seno::nuevo(sr, 0.0))).collect();
        let suma: f32 = parciales.iter().map(|p| p.1).sum();
        Serie { base_hz, parciales, norma: 1.0 / suma.max(1e-3) }
    }
}

impl Fuente for Serie {
    fn procesar(&mut self, _t0: f64, out: &mut [Cuadro]) {
        for c in out.iter_mut() {
            let mut y = 0.0;
            for (n, a, osc) in self.parciales.iter_mut() {
                y += osc.siguiente(self.base_hz * *n) * *a;
            }
            y *= self.norma;
            *c = [y, y];
        }
    }
}
