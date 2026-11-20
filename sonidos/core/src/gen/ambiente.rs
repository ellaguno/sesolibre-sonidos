use std::sync::Arc;

use super::{Cuadro, Fuente};
use crate::banco::Muestra;
use crate::dsp::UnPolo;

/// Grabación de campo en bucle. Dos copias a velocidades distintas (truco de Lienzo), la
/// segunda con los canales cruzados, para que el ciclo no se perciba; cada vuelta se funde
/// con su principio. Nominal: RMS ≈ 0,25, sea cual sea el nivel de la grabación.
pub struct Ambiente {
    m: Arc<Muestra>,
    pos: [f64; 2],
    vel: [f64; 2],
    cruce: f64,
    largo: f64,
    ganancia: f32,
    unico: bool,
    lp: Option<[UnPolo; 2]>,
}

impl Ambiente {
    /// `fiel`: las dos copias van a velocidad real (un canto no se puede bajar de tono).
    /// `unico`: una sola copia (música ya compuesta, que no debe pisarse a sí misma).
    pub fn nuevo(m: Arc<Muestra>, paso_bajo_hz: Option<f32>, fiel: bool, unico: bool, sr: f32) -> Self {
        let cruce = (2.0 * sr as f64).min(m.cuadros as f64 / 4.0).max(1.0);
        let largo = (m.cuadros as f64 - cruce).max(1.0);
        let ganancia = (0.25 / m.rms.max(1e-4)).min(8.0) * if unico { 1.0 } else { std::f32::consts::FRAC_1_SQRT_2 };
        Ambiente {
            pos: [0.0, largo * if fiel { 0.5 } else { 0.37 }],
            vel: [1.0, if fiel { 1.0 } else { 0.84 }],
            cruce,
            largo,
            ganancia,
            unico,
            lp: paso_bajo_hz.map(|f| [UnPolo::paso_bajo(f, sr), UnPolo::paso_bajo(f, sr)]),
            m,
        }
    }

    #[inline]
    fn leer(&self, p: f64) -> (f32, f32) {
        let (l, r) = self.m.leer(p);
        if p >= self.cruce {
            return (l, r);
        }
        // Principio del bucle: entra mientras sale la cola de la vuelta anterior.
        let a = (p / self.cruce) as f32;
        let (cl, cr) = self.m.leer(self.largo + p);
        let (ga, gc) = (a.sqrt(), (1.0 - a).sqrt());
        (l * ga + cl * gc, r * ga + cr * gc)
    }
}

impl Fuente for Ambiente {
    fn procesar(&mut self, _t0: f64, out: &mut [Cuadro]) {
        for c in out.iter_mut() {
            let (al, ar) = self.leer(self.pos[0]);
            let (bl, br) = if self.unico { (ar, al) } else { self.leer(self.pos[1]) };
            let (al, ar) = if self.unico { (al * 0.5, ar * 0.5) } else { (al, ar) };
            let (bl, br) = if self.unico { (bl * 0.5, br * 0.5) } else { (bl, br) };
            let mut l = (al + br) * self.ganancia;
            let mut r = (ar + bl) * self.ganancia;
            if let Some(lp) = &mut self.lp {
                l = lp[0].procesar(l);
                r = lp[1].procesar(r);
            }
            *c = [l, r];
            for i in 0..2 {
                self.pos[i] += self.vel[i];
                if self.pos[i] >= self.largo {
                    self.pos[i] -= self.largo;
                }
            }
        }
    }
}
