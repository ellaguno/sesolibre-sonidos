//! Naturaleza sintetizada: viento, lluvia y pájaros hechos con ruido filtrado y senos.
//! No son grabaciones (esas van por `ambiente`); son deterministas, no pesan nada y no se
//! repiten nunca. Nominal: RMS ≈ 0,2 (viento y lluvia); los pájaros, pico ≈ 0,6.

use std::f32::consts::{PI, TAU};

use super::{Cuadro, Fuente};
use crate::dsp::{Biquad, Seno, UnPolo};
use crate::receta::ClaseNatural;
use crate::rng::Rng;

pub fn nueva(clase: ClaseNatural, sr: f32, rng: Rng) -> Box<dyn Fuente> {
    match clase {
        ClaseNatural::Viento => Box::new(Viento::nuevo(sr, rng)),
        ClaseNatural::Lluvia => Box::new(Lluvia::nueva(sr, rng)),
        ClaseNatural::Aves => Box::new(Aves::nuevas(sr, rng)),
    }
}

/// Deriva lenta y suave en [0, 1]: tres senos de periodos que no casan.
struct Deriva {
    fases: [f32; 3],
    hz: [f32; 3],
}

impl Deriva {
    fn nueva(rng: &mut Rng, periodo_s: f32) -> Self {
        Deriva {
            fases: [rng.f32(), rng.f32(), rng.f32()],
            hz: [1.0 / periodo_s, 1.0 / (periodo_s * 0.41), 1.0 / (periodo_s * 0.173)],
        }
    }
    fn en(&self, t: f64) -> f32 {
        let s = |i: usize| ((t * self.hz[i] as f64 + self.fases[i] as f64).fract() as f32 * TAU).sin();
        (0.5 + 0.28 * s(0) + 0.15 * s(1) + 0.07 * s(2)).clamp(0.0, 1.0)
    }
}

// ------------------------------------------------------------------ viento

/// Ruido por un paso banda cuya frecuencia y fuerza vagan: rachas que suben y silban.
struct Viento {
    sr: f32,
    rng: Rng,
    tono: [Deriva; 2],
    racha: Deriva,
    filtros: [Biquad; 2],
    grave: [UnPolo; 2],
    cuenta: usize,
    amp: f32,
}

impl Viento {
    fn nuevo(sr: f32, mut rng: Rng) -> Self {
        Viento {
            sr,
            tono: [Deriva::nueva(&mut rng, 23.0), Deriva::nueva(&mut rng, 29.0)],
            racha: Deriva::nueva(&mut rng, 37.0),
            rng,
            filtros: [Biquad::paso_banda(400.0, 1.2, sr), Biquad::paso_banda(400.0, 1.2, sr)],
            grave: [UnPolo::paso_bajo(180.0, sr), UnPolo::paso_bajo(180.0, sr)],
            cuenta: 0,
            amp: 0.5,
        }
    }
}

impl Fuente for Viento {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        for (i, c) in out.iter_mut().enumerate() {
            if self.cuenta == 0 {
                // Cada 64 muestras se mueven los filtros (sin perder su estado).
                let t = t0 + i as f64 / self.sr as f64;
                let fuerza = self.racha.en(t);
                for k in 0..2 {
                    let hz = 220.0 * (1.0 + 3.2 * self.tono[k].en(t) * (0.4 + 0.6 * fuerza));
                    self.filtros[k].reafinar_banda(hz, 1.1 + 1.4 * fuerza, self.sr);
                }
                self.amp = 0.35 + 0.65 * fuerza * fuerza;
                self.cuenta = 64;
            }
            self.cuenta -= 1;
            for k in 0..2 {
                let x = self.rng.bipolar();
                let silbo = self.filtros[k].procesar(x) * 1.9;
                let cuerpo = self.grave[k].procesar(x) * 2.4;
                c[k] = (silbo * self.amp + cuerpo * 0.5) * 0.62;
            }
        }
    }
}

// ------------------------------------------------------------------ lluvia

const GOTAS: usize = 12;

/// Un siseo parejo (las miles de gotas lejanas) y gotas cercanas: impulsos que excitan
/// resonadores de frecuencia al azar, cada uno en su sitio del estéreo.
struct Lluvia {
    sr: f32,
    rng: Rng,
    alto: [UnPolo; 2],
    bajo: [UnPolo; 2],
    gotas: Vec<(Biquad, f32)>, // (resonador, paneo)
    siguiente: usize,
    por_muestra: f32,
    fuerza: Deriva,
}

impl Lluvia {
    fn nueva(sr: f32, mut rng: Rng) -> Self {
        Lluvia {
            sr,
            alto: [UnPolo::paso_alto(900.0, sr), UnPolo::paso_alto(900.0, sr)],
            bajo: [UnPolo::paso_bajo(6500.0, sr), UnPolo::paso_bajo(6500.0, sr)],
            gotas: (0..GOTAS).map(|_| (Biquad::paso_banda(3000.0, 9.0, sr), 0.0)).collect(),
            siguiente: 0,
            por_muestra: 45.0 / sr,
            fuerza: Deriva::nueva(&mut rng, 53.0),
            rng,
        }
    }
}

impl Fuente for Lluvia {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let fuerza = 0.6 + 0.4 * self.fuerza.en(t0);
        for c in out.iter_mut() {
            if self.rng.f32() < self.por_muestra * fuerza {
                let hz = 1400.0 * (1.0 + 3.5 * self.rng.f32() * self.rng.f32());
                let g = &mut self.gotas[self.siguiente];
                g.0 = Biquad::paso_banda(hz, 7.0 + 8.0 * self.rng.f32(), self.sr);
                g.1 = self.rng.bipolar() * 0.8;
                let golpe = (0.3 + 0.7 * self.rng.f32()) * 14.0;
                g.0.procesar(golpe);
                self.siguiente = (self.siguiente + 1) % GOTAS;
            }
            let (mut l, mut r) = (0.0, 0.0);
            for (res, pan) in self.gotas.iter_mut() {
                let y = res.procesar(0.0);
                l += y * (1.0 - *pan) * 0.5;
                r += y * (1.0 + *pan) * 0.5;
            }
            for (k, lado) in [l, r].into_iter().enumerate() {
                let siseo = self.bajo[k].procesar(self.alto[k].procesar(self.rng.bipolar())) * 0.42 * fuerza;
                c[k] = siseo + lado * 0.55;
            }
        }
    }
}

// ------------------------------------------------------------------ aves

/// Tres pájaros imaginarios. Cada uno, de tanto en tanto, canta una frase de silbidos que
/// suben o bajan, a veces con trino. Son senos: suenan a pajarito de caja de música, no a
/// una especie real.
struct Aves {
    sr: f32,
    rng: Rng,
    pajaros: Vec<Pajaro>,
}

struct Pajaro {
    osc: Seno,
    pan: f32,
    registro_hz: f32,
    espera: i64,       // muestras hasta el próximo silbido
    quedan: u32,       // silbidos que faltan en la frase
    pos: u32,          // muestra dentro del silbido
    largo: u32,
    f0: f32,
    f1: f32,
    trino_hz: f32,
    fuerza: f32,
}

impl Aves {
    fn nuevas(sr: f32, mut rng: Rng) -> Self {
        let pajaros = (0..3)
            .map(|i| Pajaro {
                osc: Seno::nuevo(sr, 0.0),
                pan: [-0.6, 0.1, 0.65][i],
                registro_hz: [2300.0, 3400.0, 4600.0][i],
                espera: (rng.rango(1.0, 6.0) * sr) as i64,
                quedan: 0,
                pos: 0,
                largo: 0,
                f0: 0.0,
                f1: 0.0,
                trino_hz: 0.0,
                fuerza: 0.0,
            })
            .collect();
        Aves { sr, rng, pajaros }
    }
}

impl Fuente for Aves {
    fn procesar(&mut self, _t0: f64, out: &mut [Cuadro]) {
        let sr = self.sr;
        for c in out.iter_mut() {
            let (mut l, mut r) = (0.0, 0.0);
            for p in self.pajaros.iter_mut() {
                if p.largo == 0 {
                    p.espera -= 1;
                    if p.espera > 0 {
                        continue;
                    }
                    if p.quedan == 0 {
                        // Frase nueva.
                        p.quedan = 2 + (self.rng.f32() * 5.0) as u32;
                        p.fuerza = self.rng.rango(0.35, 1.0);
                        p.trino_hz = if self.rng.f32() < 0.3 { self.rng.rango(22.0, 38.0) } else { 0.0 };
                    }
                    p.largo = (self.rng.rango(0.05, 0.19) * sr) as u32;
                    p.pos = 0;
                    p.f0 = p.registro_hz * self.rng.rango(0.8, 1.3);
                    p.f1 = p.f0 * self.rng.rango(0.72, 1.4);
                }
                let x = p.pos as f32 / p.largo as f32;
                let hz = p.f0 + (p.f1 - p.f0) * x * x;
                let mut env = (PI * x).sin();
                env *= env;
                if p.trino_hz > 0.0 {
                    env *= 0.55 + 0.45 * (TAU * p.trino_hz * p.pos as f32 / sr).sin();
                }
                let y = p.osc.siguiente(hz) * env * p.fuerza * 0.6;
                l += y * ((1.0 - p.pan) * 0.5).sqrt();
                r += y * ((1.0 + p.pan) * 0.5).sqrt();
                p.pos += 1;
                if p.pos >= p.largo {
                    p.largo = 0;
                    p.quedan -= 1;
                    p.espera = if p.quedan > 0 {
                        (self.rng.rango(0.04, 0.22) * sr) as i64
                    } else {
                        (self.rng.rango(2.5, 11.0) * sr) as i64
                    };
                }
            }
            *c = [l, r];
        }
    }
}
