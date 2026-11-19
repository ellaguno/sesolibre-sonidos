//! El motor: una receta sonando. Se le piden bloques; no sabe de hilos ni de tarjetas.

use crate::dsp::lin_a_db;
use crate::gen::{self, Cuadro, Fuente};
use crate::mezcla::{CapaMezcla, Master};
use crate::receta::Receta;
use crate::rng::Rng;
use crate::{Error, SR};

const BLOQUE: usize = 256;

pub struct Motor {
    sr: f32,
    cuadro: u64,
    total: u64,
    capas: Vec<(Box<dyn Fuente>, CapaMezcla)>,
    master: Master,
    scratch: Vec<Cuadro>,
    receta: Receta,
}

impl Motor {
    /// Crea el motor para la receta con la semilla de la receta (o la que se pase).
    pub fn nuevo(receta: &Receta, semilla: Option<u64>) -> Result<Motor, Error> {
        receta.validar()?;
        let sr = SR as f32;
        let raiz = Rng::nueva(semilla.unwrap_or(receta.semilla));
        let mut capas = Vec::new();
        for (i, capa) in receta.capas.iter().enumerate() {
            if let Some(f) = gen::construir(&capa.gen, sr, raiz.derivar(i as u64 + 1), receta.duracion_s) {
                capas.push((f, CapaMezcla::nueva(capa, receta, sr)));
            }
        }
        Ok(Motor {
            sr,
            cuadro: 0,
            total: (receta.duracion_s as f64 * SR as f64) as u64,
            capas,
            master: Master::nuevo(receta, sr),
            scratch: vec![[0.0; 2]; BLOQUE],
            receta: receta.clone(),
        })
    }

    pub fn receta(&self) -> &Receta {
        &self.receta
    }

    pub fn segundos(&self) -> f64 {
        self.cuadro as f64 / self.sr as f64
    }

    pub fn terminado(&self) -> bool {
        self.cuadro >= self.total
    }

    /// Llena `out` (estéreo intercalado L R L R…). Pasado el final, escribe silencio.
    pub fn procesar(&mut self, out: &mut [f32]) {
        let mut cuadros = out.len() / 2;
        let mut offset = 0;
        while cuadros > 0 {
            let n = cuadros.min(BLOQUE);
            self.bloque(&mut out[offset..offset + n * 2]);
            offset += n * 2;
            cuadros -= n;
        }
    }

    fn bloque(&mut self, out: &mut [f32]) {
        let n = out.len() / 2;
        out.iter_mut().for_each(|x| *x = 0.0);
        if self.terminado() {
            return;
        }
        let t0 = self.segundos();
        let dt = 1.0 / self.sr;
        let mut acum = vec![[0.0f32; 2]; n];
        for (fuente, mezcla) in self.capas.iter_mut() {
            let s = &mut self.scratch[..n];
            fuente.procesar(t0, s);
            for (i, c) in s.iter().enumerate() {
                let g = mezcla.ganancia(t0 as f32 + i as f32 * dt);
                acum[i][0] += c[0] * g;
                acum[i][1] += c[1] * g;
            }
        }
        for (i, a) in acum.iter().enumerate() {
            let t = t0 as f32 + i as f32 * dt;
            let (l, r) = self.master.procesar(t, a[0], a[1]);
            out[i * 2] = l;
            out[i * 2 + 1] = r;
        }
        self.cuadro += n as u64;
    }

    /// Rinde `segundos` (o la receta completa) a memoria. Para pruebas y para el CLI.
    pub fn rendir(&mut self, segundos: Option<f32>) -> Vec<f32> {
        let total = segundos.map(|s| (s as f64 * self.sr as f64) as u64).unwrap_or(self.total).min(self.total);
        let mut out = vec![0.0f32; total as usize * 2];
        self.procesar(&mut out);
        out
    }
}

/// Medidas para encontrar bugs, nunca para decidir si algo es bello.
#[derive(Debug, Clone, PartialEq)]
pub struct Medidas {
    pub segundos: f32,
    pub pico_db: f32,
    pub rms_db: f32,
    /// Mayor subida del nivel RMS entre ventanas consecutivas de 100 ms, en dB.
    pub salto_max_db: f32,
    pub tiene_nan: bool,
}

pub fn medir(muestras: &[f32], sr: u32) -> Medidas {
    let mut pico = 0.0f32;
    let mut suma = 0.0f64;
    let mut nan = false;
    for &x in muestras {
        if x.is_nan() {
            nan = true;
            continue;
        }
        pico = pico.max(x.abs());
        suma += (x as f64) * (x as f64);
    }
    let rms = if muestras.is_empty() { 0.0 } else { (suma / muestras.len() as f64).sqrt() as f32 };
    let ventana = (sr as usize / 10) * 2;
    let mut salto = 0.0f32;
    let mut anterior: Option<f32> = None;
    for w in muestras.chunks(ventana.max(2)) {
        let r = (w.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / w.len() as f64).sqrt() as f32;
        let db = lin_a_db(r);
        if let Some(a) = anterior {
            if db > -60.0 && a > -60.0 {
                salto = salto.max(db - a);
            }
        }
        anterior = Some(db);
    }
    Medidas { segundos: muestras.len() as f32 / 2.0 / sr as f32, pico_db: lin_a_db(pico), rms_db: lin_a_db(rms), salto_max_db: salto, tiene_nan: nan }
}
