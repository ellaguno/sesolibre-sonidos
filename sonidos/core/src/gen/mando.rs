use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use super::{Cuadro, Fuente};
use crate::dsp::Seno;

/// Un tono puro cuya frecuencia se mueve en vivo desde fuera (`Motor::mando`): el
/// generador de frecuencias del laboratorio de cimática. La frecuencia se desliza en unos
/// 60 ms para que girar el mando no haga clics. Nominal: pico ≈ 1.
pub struct Mando {
    objetivo: Arc<AtomicU32>,
    hz: f32,
    osc: Seno,
    paso: f32,
}

impl Mando {
    pub fn nuevo(objetivo: Arc<AtomicU32>, sr: f32) -> Self {
        let hz = f32::from_bits(objetivo.load(Ordering::Relaxed));
        Mando { objetivo, hz, osc: Seno::nuevo(sr, 0.0), paso: 1.0 - (-1.0 / (0.02 * sr)).exp() }
    }
}

impl Fuente for Mando {
    fn procesar(&mut self, _t0: f64, out: &mut [Cuadro]) {
        let destino = f32::from_bits(self.objetivo.load(Ordering::Relaxed));
        for c in out.iter_mut() {
            self.hz += (destino - self.hz) * self.paso;
            let y = self.osc.siguiente(self.hz);
            *c = [y, y];
        }
    }
}
