use std::sync::Arc;

use super::{Cuadro, Fuente};
use crate::banco::Muestra;
use crate::rng::Rng;

const VOCES: usize = 3;

/// Golpes sueltos y espaciados: un gong, un platillo frotado, un carillón. Cada tanto suena
/// uno entero, elegido al azar, en un sitio distinto del estéreo y con fuerza distinta.
/// Nominal: pico ≈ 0,7.
pub struct Eventos {
    sr: f32,
    rng: Rng,
    sonidos: Vec<Arc<Muestra>>,
    cada_s: f32,
    proximo_s: f64,
    ultimo: usize,
    voces: Vec<Voz>,
}

struct Voz {
    m: Arc<Muestra>,
    pos: usize,
    gl: f32,
    gr: f32,
    activa: bool,
}

impl Eventos {
    pub fn nuevos(sonidos: Vec<Arc<Muestra>>, cada_s: f32, sr: f32, mut rng: Rng) -> Self {
        let vacia = Arc::new(Muestra::nueva(1, Vec::new()));
        Eventos {
            sr,
            proximo_s: (cada_s * rng.rango(0.15, 0.4)) as f64,
            rng,
            sonidos,
            cada_s,
            ultimo: usize::MAX,
            voces: (0..VOCES).map(|_| Voz { m: vacia.clone(), pos: 0, gl: 0.0, gr: 0.0, activa: false }).collect(),
        }
    }
}

impl Fuente for Eventos {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        // El ataque se suaviza 40 ms: un gong de madrugada no debe sobresaltar.
        let ataque = (0.04 * self.sr) as usize;
        for (i, c) in out.iter_mut().enumerate() {
            let t = t0 + i as f64 * dt;
            if t >= self.proximo_s {
                self.proximo_s = t + (self.cada_s * self.rng.rango(0.6, 1.5)) as f64;
                if let Some(v) = self.voces.iter_mut().find(|v| !v.activa) {
                    let mut k = (self.rng.f32() * self.sonidos.len() as f32) as usize % self.sonidos.len();
                    if k == self.ultimo && self.sonidos.len() > 1 {
                        k = (k + 1) % self.sonidos.len();
                    }
                    self.ultimo = k;
                    let fuerza = self.rng.rango(0.45, 0.8);
                    let pan = self.rng.rango(-0.7, 0.7);
                    v.m = self.sonidos[k].clone();
                    v.pos = 0;
                    v.gl = fuerza * ((1.0 - pan) * 0.5).sqrt() * 1.4;
                    v.gr = fuerza * ((1.0 + pan) * 0.5).sqrt() * 1.4;
                    v.activa = true;
                }
            }
            let (mut l, mut r) = (0.0, 0.0);
            for v in self.voces.iter_mut().filter(|v| v.activa) {
                let (vl, vr) = v.m.leer(v.pos as f64);
                let env = (v.pos as f32 / ataque as f32).min(1.0);
                l += vl * env * v.gl;
                r += vr * env * v.gr;
                v.pos += 1;
                if v.pos >= v.m.cuadros {
                    v.activa = false;
                }
            }
            *c = [l, r];
        }
    }
}
