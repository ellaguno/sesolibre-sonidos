use super::{Cuadro, Fuente};
use crate::dsp::Seno;

/// Barrido exponencial ida y vuelta entre dos frecuencias. Solo Exploración (cimática).
pub struct Barrido {
    desde: f32,
    hasta: f32,
    periodo: f32,
    osc: Seno,
    sr: f32,
}

impl Barrido {
    pub fn nuevo(desde: f32, hasta: f32, periodo: f32, sr: f32) -> Self {
        Barrido { desde, hasta, periodo, osc: Seno::nuevo(sr, 0.0), sr }
    }
}

impl Fuente for Barrido {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        let ratio = (self.hasta / self.desde).ln();
        for (i, c) in out.iter_mut().enumerate() {
            let t = (t0 + i as f64 * dt) as f32;
            let u = (t / self.periodo).rem_euclid(1.0);
            let tri = if u < 0.5 { u * 2.0 } else { 2.0 - u * 2.0 };
            let hz = self.desde * (ratio * tri).exp();
            let y = self.osc.siguiente(hz);
            *c = [y, y];
        }
    }
}
