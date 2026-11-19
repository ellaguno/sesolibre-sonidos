use std::f32::consts::PI;
use std::sync::Arc;

use super::{Cuadro, Fuente};
use crate::banco::{mas_cercana, Muestra};
use crate::dsp::{suave, Seno};
use crate::rng::Rng;
use crate::teoria;

/// Colchón armónico: un ciclo lento de acordes que se cruzan en fundido entre dos bancos
/// de voces. Cada voz respira despacio. Dos timbres:
///
/// - **sintético**: dos osciladores desafinados (unísono ancho) y parciales suaves;
/// - **muestras**: una sección de cuerdas real sostenida sin fin, leyendo la grabación en
///   granos largos solapados (dos ventanas de Hann a medio paso suman siempre 1).
///
/// Nominal: RMS ≈ 0,15 con cinco voces.
pub struct Colchon {
    sr: f32,
    acordes: Vec<Vec<f32>>,
    seg_s: f32,
    cruce_s: f32,
    bancos: [Vec<Voz>; 2],
    acorde_actual: usize,
    brillo: f32,
    norma: f32,
    notas: Option<Vec<(f32, Arc<Muestra>)>>,
    rng: Rng,
}

struct Voz {
    hz: f32,
    cuerpo: Cuerpo,
    lfo_hz: f32,
    lfo_fase: f32,
    pan: f32,
}

enum Cuerpo {
    Sintetico { a: Seno, b: Seno, p2: Seno, p3: Seno, desaf: f32 },
    Muestra { granos: [Grano; 2] },
}

struct Grano {
    m: Arc<Muestra>,
    pos: f64,
    vel: f64,
    fase: f32,
    paso: f32,
}

impl Grano {
    /// Empieza una ventana nueva en un punto al azar de la parte sostenida de la grabación.
    fn renovar(&mut self, notas: &[(f32, Arc<Muestra>)], hz: f32, sr: f32, rng: &mut Rng) {
        let (hz_muestra, m) = mas_cercana(notas, hz);
        self.m = m.clone();
        self.vel = (hz / hz_muestra) as f64;
        let largo = m.cuadros as f64;
        // Se evita el ataque (primer 15 %) y la caída (último 20 %).
        let (ini, fin) = (largo * 0.15, largo * 0.80);
        let ventana_s = (4.0f64).min((fin - ini) * 0.85 / self.vel / sr as f64).max(0.25);
        let recorrido = ventana_s * sr as f64 * self.vel;
        self.pos = ini + rng.f32() as f64 * (fin - ini - recorrido).max(0.0);
        self.paso = 1.0 / (ventana_s as f32 * sr);
    }
}

impl Colchon {
    #[allow(clippy::too_many_arguments)]
    pub fn nuevo(
        raiz_hz: f32,
        modo: &str,
        ciclo_s: f32,
        voces: usize,
        brillo: f32,
        notas: Option<Vec<(f32, Arc<Muestra>)>>,
        sr: f32,
        mut rng: Rng,
    ) -> Self {
        let acordes = teoria::acordes_hz(raiz_hz, modo, voces);
        let seg_s = teoria::segmento_s(ciclo_s, acordes.len());
        let cruce_s = (seg_s * 0.5).min(12.0);
        let hacer = |hzs: &[f32], rng: &mut Rng| -> Vec<Voz> {
            hzs.iter()
                .enumerate()
                .map(|(i, &hz)| {
                    let cuerpo = match &notas {
                        None => Cuerpo::Sintetico {
                            a: Seno::nuevo(sr, rng.f32()),
                            b: Seno::nuevo(sr, rng.f32()),
                            p2: Seno::nuevo(sr, rng.f32()),
                            p3: Seno::nuevo(sr, rng.f32()),
                            desaf: rng.rango(0.0012, 0.0025),
                        },
                        Some(n) => {
                            let g = |fase: f32, rng: &mut Rng| {
                                let mut g = Grano { m: n[0].1.clone(), pos: 0.0, vel: 1.0, fase, paso: 0.0 };
                                g.renovar(n, hz, sr, rng);
                                g
                            };
                            let desfase = rng.f32() * 0.5;
                            Cuerpo::Muestra { granos: [g(desfase, rng), g(desfase + 0.5, rng)] }
                        }
                    };
                    Voz {
                        hz,
                        cuerpo,
                        lfo_hz: rng.rango(0.04, 0.12),
                        lfo_fase: rng.f32(),
                        pan: if i % 2 == 0 { -0.35 } else { 0.35 } * rng.rango(0.5, 1.0),
                    }
                })
                .collect()
        };
        let b0 = hacer(&acordes[0], &mut rng);
        let b1 = hacer(&acordes[1 % acordes.len()], &mut rng);
        let n = voces.max(1) as f32;
        // Sintético: cada voz tiene pico ≈ 1. Muestras: cada grabación tiene RMS ≈ 0,1.
        let norma = if notas.is_some() { 3.5 } else { 0.7 } / n.sqrt();
        Colchon { sr, acordes, seg_s, cruce_s, bancos: [b0, b1], acorde_actual: 0, brillo: brillo.clamp(0.0, 1.0), norma, notas, rng }
    }

    /// Cambia el acorde de un banco. Se llama cuando ese banco está en silencio.
    fn retonar(&mut self, banco: usize, acorde: usize) {
        let hzs = &self.acordes[acorde % self.acordes.len()];
        for (v, &hz) in self.bancos[banco].iter_mut().zip(hzs) {
            v.hz = hz;
            if let (Cuerpo::Muestra { granos }, Some(notas)) = (&mut v.cuerpo, &self.notas) {
                for g in granos.iter_mut() {
                    g.renovar(notas, hz, self.sr, &mut self.rng);
                }
            }
        }
    }
}

impl Voz {
    #[inline]
    fn muestra(&mut self, sr: f32, brillo: f32, notas: Option<&[(f32, Arc<Muestra>)]>, rng: &mut Rng) -> (f32, f32) {
        let hz = self.hz;
        let (mut l, mut r) = match &mut self.cuerpo {
            Cuerpo::Sintetico { a, b, p2, p3, desaf } => {
                let y = 0.5 * (a.siguiente(hz * (1.0 - *desaf)) + b.siguiente(hz * (1.0 + *desaf)))
                    + brillo * (0.22 * p2.siguiente(hz * 2.0) + 0.08 * p3.siguiente(hz * 3.0));
                (y, y)
            }
            Cuerpo::Muestra { granos } => {
                let (mut l, mut r) = (0.0, 0.0);
                for g in granos.iter_mut() {
                    let w = 0.5 - 0.5 * (2.0 * PI * g.fase).cos();
                    let (gl, gr) = g.m.leer(g.pos);
                    l += gl * w;
                    r += gr * w;
                    g.pos += g.vel;
                    g.fase += g.paso;
                    if g.fase >= 1.0 {
                        g.fase -= 1.0;
                        if let Some(n) = notas {
                            g.renovar(n, hz, sr, rng);
                        }
                    }
                }
                (l, r)
            }
        };
        self.lfo_fase += self.lfo_hz / sr;
        if self.lfo_fase >= 1.0 {
            self.lfo_fase -= 1.0;
        }
        let resp = 0.75 + 0.25 * (2.0 * PI * self.lfo_fase).sin();
        l *= resp * ((1.0 - self.pan) * 0.5).sqrt();
        r *= resp * ((1.0 + self.pan) * 0.5).sqrt();
        (l, r)
    }
}

impl Fuente for Colchon {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        for (i, c) in out.iter_mut().enumerate() {
            let t = (t0 + i as f64 * dt) as f32;
            let idx = (t / self.seg_s) as usize;
            if idx != self.acorde_actual {
                // El acorde nuevo entra por el banco que ya terminó de salir.
                self.acorde_actual = idx;
                self.retonar(idx % 2, idx);
            }
            // El banco idx%2 lleva el acorde actual; el otro, el anterior, y se apaga en
            // `cruce_s` desde el inicio del segmento.
            let en_seg = t - idx as f32 * self.seg_s;
            let x = if idx == 0 { 1.0 } else { suave(en_seg / self.cruce_s) };
            let actual = idx % 2;
            let (mut l, mut r) = (0.0, 0.0);
            let notas = self.notas.as_deref();
            for (bi, banco) in self.bancos.iter_mut().enumerate() {
                let g = if bi == actual { x } else { 1.0 - x };
                if g <= 0.0 {
                    continue;
                }
                for v in banco.iter_mut() {
                    let (vl, vr) = v.muestra(self.sr, self.brillo, notas, &mut self.rng);
                    l += vl * g;
                    r += vr * g;
                }
            }
            *c = [l * self.norma, r * self.norma];
        }
    }
}
