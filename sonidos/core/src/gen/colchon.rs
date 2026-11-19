use std::f32::consts::PI;

use super::{Cuadro, Fuente};
use crate::dsp::{suave, Seno};
use crate::rng::Rng;
use crate::teoria;

/// Colchón armónico: un ciclo lento de acordes, cada voz con dos osciladores desafinados
/// (unísono ancho), parciales suaves y una respiración lenta de amplitud. Los acordes se
/// cruzan en fundido entre dos bancos. Nominal: pico ≈ 0,7.
pub struct Colchon {
    sr: f32,
    acordes: Vec<Vec<f32>>, // frecuencias por acorde
    seg_s: f32,             // duración de cada acorde
    cruce_s: f32,
    bancos: [Banco; 2],
    acorde_actual: usize,
    brillo: f32,
    norma: f32,
}

struct Banco {
    voces: Vec<Voz>,
}

struct Voz {
    hz: f32,
    a: Seno,
    b: Seno,
    p2: Seno,
    p3: Seno,
    desaf: f32,
    lfo_hz: f32,
    lfo_fase: f32,
    pan: f32,
}

impl Colchon {
    pub fn nuevo(raiz_hz: f32, modo: &str, ciclo_s: f32, voces: usize, brillo: f32, sr: f32, mut rng: Rng) -> Self {
        let ciclo = teoria::ciclo_de_acordes(modo);
        let acordes: Vec<Vec<f32>> = ciclo
            .iter()
            .map(|ac| ac.iter().take(voces).map(|&st| teoria::semitonos(raiz_hz, st)).collect())
            .collect();
        let seg_s = (ciclo_s / acordes.len() as f32).max(10.0);
        let cruce_s = (seg_s * 0.5).min(12.0);
        let hacer_banco = |rng: &mut Rng, hzs: &[f32]| Banco {
            voces: hzs
                .iter()
                .enumerate()
                .map(|(i, &hz)| Voz {
                    hz,
                    a: Seno::nuevo(sr, rng.f32()),
                    b: Seno::nuevo(sr, rng.f32()),
                    p2: Seno::nuevo(sr, rng.f32()),
                    p3: Seno::nuevo(sr, rng.f32()),
                    desaf: rng.rango(0.0012, 0.0025),
                    lfo_hz: rng.rango(0.04, 0.12),
                    lfo_fase: rng.f32(),
                    pan: if i % 2 == 0 { -0.35 } else { 0.35 } * rng.rango(0.5, 1.0),
                })
                .collect(),
        };
        let b0 = hacer_banco(&mut rng, &acordes[0]);
        let b1 = hacer_banco(&mut rng, &acordes[1 % acordes.len()]);
        let n = voces.max(1) as f32;
        Colchon { sr, acordes, seg_s, cruce_s, bancos: [b0, b1], acorde_actual: 0, brillo: brillo.clamp(0.0, 1.0), norma: 0.7 / n.sqrt() }
    }

    fn retonar(&mut self, banco: usize, acorde: usize) {
        let hzs = &self.acordes[acorde % self.acordes.len()];
        for (v, &hz) in self.bancos[banco].voces.iter_mut().zip(hzs) {
            v.hz = hz;
        }
    }
}

impl Voz {
    #[inline]
    fn muestra(&mut self, sr: f32, brillo: f32) -> (f32, f32) {
        let hz = self.hz;
        let y = 0.5 * (self.a.siguiente(hz * (1.0 - self.desaf)) + self.b.siguiente(hz * (1.0 + self.desaf)))
            + brillo * (0.22 * self.p2.siguiente(hz * 2.0) + 0.08 * self.p3.siguiente(hz * 3.0));
        self.lfo_fase += self.lfo_hz / sr;
        if self.lfo_fase >= 1.0 {
            self.lfo_fase -= 1.0;
        }
        let resp = 0.75 + 0.25 * (2.0 * PI * self.lfo_fase).sin();
        let y = y * resp;
        let gl = ((1.0 - self.pan) * 0.5).sqrt();
        let gr = ((1.0 + self.pan) * 0.5).sqrt();
        (y * gl, y * gr)
    }
}

impl Fuente for Colchon {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        for (i, c) in out.iter_mut().enumerate() {
            let t = (t0 + i as f64 * dt) as f32;
            let idx = (t / self.seg_s) as usize;
            if idx != self.acorde_actual {
                // Entra el acorde idx en el banco que estaba saliendo.
                self.acorde_actual = idx;
                let entrante = idx % 2;
                self.retonar(entrante, idx);
            }
            // Mezcla entre bancos: el banco idx%2 lleva el acorde actual; el otro, el anterior,
            // y se apaga en `cruce_s` desde el inicio del segmento.
            let en_seg = t - idx as f32 * self.seg_s;
            let x = suave(en_seg / self.cruce_s);
            let actual = idx % 2;
            let (mut l, mut r) = (0.0, 0.0);
            for (bi, banco) in self.bancos.iter_mut().enumerate() {
                let g = if bi == actual { x } else { 1.0 - x };
                if g <= 0.0 {
                    continue;
                }
                for v in banco.voces.iter_mut() {
                    let (vl, vr) = v.muestra(self.sr, self.brillo);
                    l += vl * g;
                    r += vr * g;
                }
            }
            *c = [l * self.norma, r * self.norma];
        }
    }
}
