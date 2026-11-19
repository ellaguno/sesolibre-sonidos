//! El motor: una receta sonando. Se le piden bloques; no sabe de hilos ni de tarjetas.
//! No reserva memoria al procesar: puede vivir en un hilo de audio.

use crate::banco::Banco;
use crate::dsp::{db_a_lin, lin_a_db};
use crate::espacio::Reverb;
use crate::gen::{self, Cuadro, Fuente};
use crate::mezcla::{CapaMezcla, Master};
use crate::receta::Receta;
use crate::rng::Rng;
use crate::{Error, SR};

const BLOQUE: usize = 256;
/// Lo más que puede durar una sesión, se alargue lo que se alargue.
const MAXIMO_S: f64 = 12.0 * 3600.0;

struct CapaActiva {
    fuente: Box<dyn Fuente>,
    mezcla: CapaMezcla,
    envio: f32,
}

pub struct Motor {
    sr: f32,
    cuadro: u64,
    total: u64,
    capas: Vec<CapaActiva>,
    master: Master,
    reverb: Option<Reverb>,
    scratch: Vec<Cuadro>,
    acum: Vec<Cuadro>,
    envios: Vec<Cuadro>,
    receta: Receta,
    /// Pausa suave: 0 = callado, 1 = sonando. Baja en 0,4 s y sube en 1,5 s.
    transporte: f32,
    atenuado: bool,
    /// Volumen del usuario (≤ 0 dB), p. ej. el modo noche. Se mueve a 12 dB/s.
    volumen_db: f32,
    volumen_objetivo_db: f32,
}

impl Motor {
    /// Una receta que no usa muestras (o una prueba). Con muestras: `con_banco`.
    pub fn nuevo(receta: &Receta, semilla: Option<u64>) -> Result<Motor, Error> {
        Motor::con_banco(receta, semilla, &Banco::vacio())
    }

    /// Crea el motor para la receta con la semilla de la receta (o la que se pase).
    pub fn con_banco(receta: &Receta, semilla: Option<u64>, banco: &Banco) -> Result<Motor, Error> {
        receta.validar()?;
        let sr = SR as f32;
        let raiz = Rng::nueva(semilla.unwrap_or(receta.semilla));
        let mut capas = Vec::new();
        for (i, capa) in receta.capas.iter().enumerate() {
            let fuente = gen::construir(&capa.gen, sr, raiz.derivar(i as u64 + 1), receta.duracion_s, banco)?;
            capas.push(CapaActiva { fuente, mezcla: CapaMezcla::nueva(capa, receta, sr), envio: capa.reverb });
        }
        let reverb = if capas.iter().any(|c| c.envio > 0.0) {
            let e = receta.espacio.unwrap_or_default();
            Some(Reverb::nueva(e.rt60_s, e.amortiguacion_hz, sr))
        } else {
            None
        };
        Ok(Motor {
            sr,
            cuadro: 0,
            total: (receta.duracion_s as f64 * SR as f64) as u64,
            capas,
            master: Master::nuevo(receta, sr),
            reverb,
            scratch: vec![[0.0; 2]; BLOQUE],
            acum: vec![[0.0; 2]; BLOQUE],
            envios: vec![[0.0; 2]; BLOQUE],
            receta: receta.clone(),
            transporte: 1.0,
            atenuado: false,
            volumen_db: 0.0,
            volumen_objetivo_db: 0.0,
        })
    }

    pub fn receta(&self) -> &Receta {
        &self.receta
    }

    pub fn segundos(&self) -> f64 {
        self.cuadro as f64 / self.sr as f64
    }

    pub fn duracion_s(&self) -> f64 {
        self.total as f64 / self.sr as f64
    }

    pub fn restante_s(&self) -> f64 {
        (self.duracion_s() - self.segundos()).max(0.0)
    }

    pub fn terminado(&self) -> bool {
        self.cuadro >= self.total
    }

    /// Pausa (o reanuda) sin clic. Tras `atenuar(true)` hay que seguir pidiendo bloques
    /// hasta que `en_silencio()`; entonces se puede dejar de pedir y el tiempo se detiene.
    pub fn atenuar(&mut self, callar: bool) {
        self.atenuado = callar;
    }

    pub fn en_silencio(&self) -> bool {
        self.atenuado && self.transporte <= 0.0
    }

    /// Volumen del usuario en dB (0 o menos). El techo de la receta no se puede subir.
    pub fn volumen(&mut self, db: f32) {
        self.volumen_objetivo_db = db.clamp(-60.0, 0.0);
    }

    /// Alarga la sesión ("+15 min"). Devuelve los segundos realmente añadidos.
    pub fn extender(&mut self, segundos: f32) -> f32 {
        let margen = (MAXIMO_S - self.duracion_s()).max(0.0) as f32;
        let s = segundos.clamp(0.0, margen);
        self.total += (s as f64 * self.sr as f64) as u64;
        self.master.extender(s);
        for c in self.capas.iter_mut() {
            c.mezcla.extender(s);
        }
        s
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
        let acum = &mut self.acum[..n];
        let envios = &mut self.envios[..n];
        acum.iter_mut().for_each(|c| *c = [0.0; 2]);
        envios.iter_mut().for_each(|c| *c = [0.0; 2]);
        for capa in self.capas.iter_mut() {
            let s = &mut self.scratch[..n];
            capa.fuente.procesar(t0, s);
            for (i, c) in s.iter().enumerate() {
                let g = capa.mezcla.ganancia(t0 as f32 + i as f32 * dt);
                acum[i][0] += c[0] * g;
                acum[i][1] += c[1] * g;
                if capa.envio > 0.0 {
                    envios[i][0] += c[0] * g * capa.envio;
                    envios[i][1] += c[1] * g * capa.envio;
                }
            }
        }
        let baja = dt / 0.4;
        let sube = dt / 1.5;
        let paso_vol = 12.0 * dt;
        for i in 0..n {
            let t = t0 as f32 + i as f32 * dt;
            let (mut l, mut r) = (acum[i][0], acum[i][1]);
            if let Some(rev) = &mut self.reverb {
                let (rl, rr) = rev.procesar(envios[i][0], envios[i][1]);
                l += rl;
                r += rr;
            }
            self.transporte = if self.atenuado { (self.transporte - baja).max(0.0) } else { (self.transporte + sube).min(1.0) };
            if self.volumen_db != self.volumen_objetivo_db {
                let d = (self.volumen_objetivo_db - self.volumen_db).clamp(-paso_vol, paso_vol);
                self.volumen_db += d;
            }
            let usuario = self.transporte * self.transporte * if self.volumen_db == 0.0 { 1.0 } else { db_a_lin(self.volumen_db) };
            let (l, r) = self.master.procesar(t, l, r);
            out[i * 2] = l * usuario;
            out[i * 2 + 1] = r * usuario;
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
