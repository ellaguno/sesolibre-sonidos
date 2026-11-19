use super::{Cuadro, Fuente};
use crate::dsp::{suave, Biquad, Seno};
use crate::rng::Rng;

/// Guía de respiración: un *swell* de aire (ruido filtrado) y un tono suave que suben al
/// inhalar, se sostienen y bajan al exhalar. Sin voz. Nominal: pico ≈ 0,8.
pub struct Respiracion {
    inhalar: f32,
    retener: f32,
    exhalar: f32,
    pausa: f32,
    hz: f32,
    sr: f32,
    rng: Rng,
    aire: [Biquad; 2],
    tono: Seno,
    tono2: Seno,
}

impl Respiracion {
    pub fn nueva(inhalar: f32, retener: f32, exhalar: f32, pausa: f32, hz: f32, sr: f32, rng: Rng) -> Self {
        Respiracion {
            inhalar,
            retener,
            exhalar,
            pausa,
            hz,
            sr,
            rng,
            aire: [Biquad::paso_banda(700.0, 0.8, sr), Biquad::paso_banda(900.0, 0.8, sr)],
            tono: Seno::nuevo(sr, 0.0),
            tono2: Seno::nuevo(sr, 0.5),
        }
    }

    /// Envolvente 0..1 y fase (0 inhalar, 1 retener, 2 exhalar, 3 pausa).
    fn envolvente(&self, t: f32) -> (f32, u8) {
        let ciclo = self.inhalar + self.retener + self.exhalar + self.pausa;
        let mut u = t.rem_euclid(ciclo);
        if u < self.inhalar {
            return (suave(u / self.inhalar), 0);
        }
        u -= self.inhalar;
        if u < self.retener {
            return (1.0, 1);
        }
        u -= self.retener;
        if u < self.exhalar {
            return (1.0 - suave(u / self.exhalar), 2);
        }
        (0.0, 3)
    }
}

impl Fuente for Respiracion {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        for (i, c) in out.iter_mut().enumerate() {
            let t = (t0 + i as f64 * dt) as f32;
            let (e, fase) = self.envolvente(t);
            // El aire suena al moverse (inhalar/exhalar), no al retener.
            let mov = if fase == 1 { 0.15 } else { 1.0 };
            let aire_amp = e.powf(1.6) * mov;
            let blanco = self.rng.bipolar();
            let al = self.aire[0].procesar(blanco) * 1.2 * aire_amp;
            let ar = self.aire[1].procesar(self.rng.bipolar()) * 1.2 * aire_amp;
            // El tono sube medio semitono al llenarse; da la sensación de "arriba".
            let hz = self.hz * (1.0 + 0.03 * e);
            let tono = (0.7 * self.tono.siguiente(hz) + 0.3 * self.tono2.siguiente(hz * 2.0)) * 0.4 * e;
            *c = [al + tono, ar + tono];
        }
    }
}
