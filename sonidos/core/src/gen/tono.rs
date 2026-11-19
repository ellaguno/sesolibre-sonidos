use super::{Cuadro, Fuente};
use crate::dsp::Seno;

/// Tono puro, con parciales opcionales a -12 dB cada uno. Nominal: pico ≈ 1.
pub struct Tono {
    hz: f32,
    oscs: Vec<Seno>,
    norma: f32,
}

impl Tono {
    pub fn nuevo(hz: f32, armonicos: u8, sr: f32) -> Self {
        let n = 1 + armonicos as usize;
        let oscs = (0..n).map(|i| Seno::nuevo(sr, i as f32 * 0.37)).collect();
        let suma: f32 = (0..n).map(|i| 0.25f32.powi(i as i32)).sum();
        Tono { hz, oscs, norma: 1.0 / suma }
    }
}

impl Fuente for Tono {
    fn procesar(&mut self, _t0: f64, out: &mut [Cuadro]) {
        for c in out.iter_mut() {
            let mut y = 0.0;
            for (i, o) in self.oscs.iter_mut().enumerate() {
                y += o.siguiente(self.hz * (i + 1) as f32) * 0.25f32.powi(i as i32);
            }
            y *= self.norma;
            *c = [y, y];
        }
    }
}
