use std::sync::Arc;

use super::{Cuadro, Fuente};
use crate::banco::{mas_cercana, Muestra};
use crate::rng::Rng;
use crate::teoria;

const VOCES: usize = 12;

/// Un instrumento real tocando solo: notas sueltas y lentas, elegidas del acorde que suena
/// en el colchón (mismo modo, raíz y ciclo → mismo acorde, sin coordinarse). Las pulsadas
/// (arpa, vibráfono) suenan hasta apagarse; las sostenidas (flauta, cuerdas) entran y salen
/// despacio. Nominal: pico ≈ 0,7.
pub struct Instrumento {
    sr: f32,
    rng: Rng,
    notas: Vec<(f32, Arc<Muestra>)>,
    sostenido: bool,
    acordes: Vec<Vec<f32>>,
    seg_s: f32,
    octavas: (i32, i32),
    intervalo_s: f32,
    proxima_s: f64,
    ultima_hz: f32,
    voces: Vec<Voz>,
}

struct Voz {
    m: Arc<Muestra>,
    pos: f64,
    vel: f64,
    gl: f32,
    gr: f32,
    edad: u32,
    ataque: u32,
    sosten: u32,
    relajo: u32,
    en_relajo: Option<u32>,
    activa: bool,
}

impl Instrumento {
    #[allow(clippy::too_many_arguments)]
    pub fn nuevo(
        notas: Vec<(f32, Arc<Muestra>)>,
        sostenido: bool,
        raiz_hz: f32,
        modo: &str,
        ciclo_s: f32,
        notas_por_minuto: f32,
        octavas: (i32, i32),
        sr: f32,
        mut rng: Rng,
    ) -> Self {
        let acordes = teoria::acordes_hz(raiz_hz, modo, 5);
        let seg_s = teoria::segmento_s(ciclo_s, acordes.len());
        let intervalo_s = 60.0 / notas_por_minuto.max(0.1);
        let vacia = Arc::new(Muestra::nueva(1, Vec::new()));
        Instrumento {
            sr,
            proxima_s: (intervalo_s * rng.rango(0.2, 0.6)) as f64,
            rng,
            notas,
            sostenido,
            acordes,
            seg_s,
            octavas,
            intervalo_s,
            ultima_hz: 0.0,
            voces: (0..VOCES)
                .map(|_| Voz { m: vacia.clone(), pos: 0.0, vel: 1.0, gl: 0.0, gr: 0.0, edad: 0, ataque: 1, sosten: 0, relajo: 1, en_relajo: None, activa: false })
                .collect(),
        }
    }

    fn elegir_hz(&mut self, t: f64) -> f32 {
        let acorde = &self.acordes[(t as f32 / self.seg_s) as usize % self.acordes.len()];
        let lo = self.notas.first().map(|n| n.0).unwrap_or(100.0) / 1.5;
        let hi = self.notas.last().map(|n| n.0).unwrap_or(1000.0) * 1.5;
        let mut candidatas: Vec<f32> = Vec::with_capacity(16);
        for &hz in acorde {
            for o in self.octavas.0..=self.octavas.1 {
                let f = hz * 2f32.powi(o);
                if f >= lo && f <= hi && (f / self.ultima_hz.max(1.0) - 1.0).abs() > 0.01 {
                    candidatas.push(f);
                }
            }
        }
        if candidatas.is_empty() {
            // El registro pedido no cabe en el instrumento: se lleva por octavas al suyo.
            let mut f = acorde[0];
            while f < lo {
                f *= 2.0;
            }
            while f > hi {
                f /= 2.0;
            }
            return f;
        }
        let i = (self.rng.f32() * candidatas.len() as f32) as usize;
        candidatas[i.min(candidatas.len() - 1)]
    }

    fn tocar(&mut self, t: f64) {
        let hz = self.elegir_hz(t);
        self.ultima_hz = hz;
        let Some(libre) = self.voces.iter().position(|v| !v.activa) else {
            return; // todas ocupadas: esta nota no suena (mejor que cortar otra con un clic)
        };
        let (hz_muestra, m) = mas_cercana(&self.notas, hz);
        let fuerza = self.rng.rango(0.45, 1.0);
        let pan = self.rng.rango(-0.6, 0.6);
        let sr = self.sr;
        let (ataque, sosten, relajo, nominal) = if self.sostenido {
            (0.7 * sr, self.rng.rango(2.5, 4.5) * sr, 2.2 * sr, 1.6)
        } else {
            (0.003 * sr, 9.0 * sr, 1.5 * sr, 0.8)
        };
        let v = &mut self.voces[libre];
        v.m = m.clone();
        v.pos = 0.0;
        v.vel = (hz / hz_muestra) as f64;
        v.gl = nominal * fuerza * ((1.0 - pan) * 0.5).sqrt();
        v.gr = nominal * fuerza * ((1.0 + pan) * 0.5).sqrt();
        v.edad = 0;
        v.ataque = ataque.max(1.0) as u32;
        v.sosten = sosten as u32;
        v.relajo = relajo.max(1.0) as u32;
        v.en_relajo = None;
        v.activa = true;
    }
}

impl Voz {
    #[inline]
    fn muestra(&mut self) -> (f32, f32) {
        let restantes = ((self.m.cuadros as f64 - self.pos) / self.vel) as i64;
        if self.en_relajo.is_none() && (self.edad >= self.sosten || restantes <= self.relajo as i64) {
            self.en_relajo = Some(self.edad);
        }
        let mut env = (self.edad as f32 / self.ataque as f32).min(1.0);
        if let Some(inicio) = self.en_relajo {
            let x = 1.0 - (self.edad - inicio) as f32 / self.relajo as f32;
            if x <= 0.0 || restantes <= 0 {
                self.activa = false;
                return (0.0, 0.0);
            }
            env *= x * x;
        }
        let (l, r) = self.m.leer(self.pos);
        self.pos += self.vel;
        self.edad += 1;
        (l * env * self.gl, r * env * self.gr)
    }
}

impl Fuente for Instrumento {
    fn procesar(&mut self, t0: f64, out: &mut [Cuadro]) {
        let dt = 1.0 / self.sr as f64;
        for (i, c) in out.iter_mut().enumerate() {
            let t = t0 + i as f64 * dt;
            if t >= self.proxima_s {
                self.tocar(t);
                self.proxima_s = t + (self.intervalo_s * self.rng.rango(0.5, 1.6)) as f64;
            }
            let (mut l, mut r) = (0.0, 0.0);
            for v in self.voces.iter_mut() {
                if v.activa {
                    let (vl, vr) = v.muestra();
                    l += vl;
                    r += vr;
                }
            }
            *c = [l, r];
        }
    }
}
