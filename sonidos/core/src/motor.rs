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
/// Muestras que mira el espectrómetro (43 ms: resuelve desde ~47 Hz).
const VENTANA: usize = 2048;
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
    /// Lo último que salió (mono), en anillo, para el espectrómetro.
    anillo: Vec<f32>,
    anillo_pos: usize,
    /// Frecuencia (bits de f32) del tono en vivo, si la receta tiene una capa `mando`.
    mando: std::sync::Arc<std::sync::atomic::AtomicU32>,
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
        let hz_inicial = receta
            .capas
            .iter()
            .find_map(|c| if let crate::receta::Generador::Mando { hz } = c.gen { Some(hz) } else { None })
            .unwrap_or(220.0);
        let mando = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(hz_inicial.to_bits()));
        let mut capas = Vec::new();
        for (i, capa) in receta.capas.iter().enumerate() {
            let fuente = gen::construir(&capa.gen, sr, raiz.derivar(i as u64 + 1), receta.duracion_s, banco, &mando)?;
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
            anillo: vec![0.0; VENTANA],
            anillo_pos: 0,
            mando,
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

    /// Mueve el tono en vivo (capa `mando`) a `hz`, dentro de 20–2000 Hz. Devuelve la
    /// frecuencia que quedó. Sin esa capa no hace nada audible.
    pub fn mando(&self, hz: f32) -> f32 {
        let hz = if hz.is_finite() { hz } else { 220.0 }.clamp(crate::receta::MANDO_MIN_HZ, crate::receta::MANDO_MAX_HZ);
        self.mando.store(hz.to_bits(), std::sync::atomic::Ordering::Relaxed);
        hz
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
            self.anillo[self.anillo_pos] = (l + r) * 0.5 * usuario;
            self.anillo_pos = (self.anillo_pos + 1) % VENTANA;
        }
        self.cuadro += n as u64;
    }

    /// El espectro de lo último que sonó, en `bandas.len()` bandas logarítmicas de 40 Hz a
    /// 16 kHz. Cada valor va de 0 (silencio, −100 dBFS) a 1 (−20 dBFS). No lo llama el hilo
    /// de audio: reserva memoria y hace una FFT.
    pub fn espectro(&self, bandas: &mut [f32]) {
        let n = VENTANA;
        let mut re = vec![0.0f32; n];
        let mut im = vec![0.0f32; n];
        for (i, x) in re.iter_mut().enumerate() {
            let w = 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / n as f32).cos();
            *x = self.anillo[(self.anillo_pos + i) % n] * w;
        }
        crate::dsp::fft(&mut re, &mut im);
        let por_bin = self.sr / n as f32;
        let (lo, hi) = (40.0f32, 16_000.0f32);
        let total = bandas.len().max(1) as f32;
        for (b, salida) in bandas.iter_mut().enumerate() {
            let f0 = lo * (hi / lo).powf(b as f32 / total);
            let f1 = lo * (hi / lo).powf((b + 1) as f32 / total);
            let i0 = ((f0 / por_bin) as usize).max(1);
            let i1 = ((f1 / por_bin).ceil() as usize).clamp(i0 + 1, n / 2);
            let mut pico = 0.0f32;
            for i in i0..i1 {
                pico = pico.max(re[i] * re[i] + im[i] * im[i]);
            }
            // Un seno a fondo de escala da n/4 tras la ventana de Hann.
            let db = lin_a_db(pico.sqrt() / (n as f32 / 4.0));
            *salida = ((db + 100.0) / 80.0).clamp(0.0, 1.0);
        }
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
