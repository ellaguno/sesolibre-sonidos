//! Piezas de DSP pequeñas y sin sorpresas.

use std::f32::consts::PI;

pub fn db_a_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn lin_a_db(lin: f32) -> f32 {
    if lin <= 1e-9 {
        -180.0
    } else {
        20.0 * lin.log10()
    }
}

/// Filtro de un polo. `lp` = paso bajo; si no, paso alto.
#[derive(Debug, Clone)]
pub struct UnPolo {
    a: f32,
    z: f32,
    lp: bool,
}

impl UnPolo {
    pub fn paso_bajo(hz: f32, sr: f32) -> Self {
        UnPolo { a: coef(hz, sr), z: 0.0, lp: true }
    }
    pub fn paso_alto(hz: f32, sr: f32) -> Self {
        UnPolo { a: coef(hz, sr), z: 0.0, lp: false }
    }
    #[inline]
    pub fn procesar(&mut self, x: f32) -> f32 {
        self.z += self.a * (x - self.z);
        if self.lp {
            self.z
        } else {
            x - self.z
        }
    }
}

fn coef(hz: f32, sr: f32) -> f32 {
    let x = (-2.0 * PI * hz.clamp(1.0, sr * 0.45) / sr).exp();
    1.0 - x
}

/// Biquad RBJ (paso bajo, paso alto, paso banda).
#[derive(Debug, Clone, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    pub fn paso_bajo(hz: f32, q: f32, sr: f32) -> Self {
        let (w, alpha) = wa(hz, q, sr);
        let c = w.cos();
        let a0 = 1.0 + alpha;
        Self::norm((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, -2.0 * c, 1.0 - alpha, a0)
    }
    pub fn paso_alto(hz: f32, q: f32, sr: f32) -> Self {
        let (w, alpha) = wa(hz, q, sr);
        let c = w.cos();
        let a0 = 1.0 + alpha;
        Self::norm((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0, -2.0 * c, 1.0 - alpha, a0)
    }
    pub fn paso_banda(hz: f32, q: f32, sr: f32) -> Self {
        let (w, alpha) = wa(hz, q, sr);
        let c = w.cos();
        let a0 = 1.0 + alpha;
        Self::norm(alpha, 0.0, -alpha, -2.0 * c, 1.0 - alpha, a0)
    }
    /// Mueve un paso banda a otra frecuencia sin perder el estado (para filtros que vagan).
    pub fn reafinar_banda(&mut self, hz: f32, q: f32, sr: f32) {
        let n = Biquad::paso_banda(hz, q, sr);
        self.b0 = n.b0;
        self.b1 = n.b1;
        self.b2 = n.b2;
        self.a1 = n.a1;
        self.a2 = n.a2;
    }
    fn norm(b0: f32, b1: f32, b2: f32, a1: f32, a2: f32, a0: f32) -> Self {
        Biquad { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0, ..Default::default() }
    }
    #[inline]
    pub fn procesar(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

fn wa(hz: f32, q: f32, sr: f32) -> (f32, f32) {
    let w = 2.0 * PI * hz.clamp(10.0, sr * 0.45) / sr;
    (w, w.sin() / (2.0 * q.max(0.1)))
}

/// Oscilador senoidal por acumulador de fase. La frecuencia puede cambiar en cada muestra
/// sin clics.
#[derive(Debug, Clone)]
pub struct Seno {
    fase: f32,
    sr: f32,
}

impl Seno {
    pub fn nuevo(sr: f32, fase_inicial: f32) -> Self {
        Seno { fase: fase_inicial.rem_euclid(1.0), sr }
    }
    #[inline]
    pub fn siguiente(&mut self, hz: f32) -> f32 {
        let y = (self.fase * 2.0 * PI).sin();
        self.fase += hz / self.sr;
        if self.fase >= 1.0 {
            self.fase -= 1.0;
        }
        y
    }
}

/// Ganancia que **nunca sube de golpe**: por encima del umbral audible la subida está
/// limitada en dB por segundo; por debajo (de -90 a -50 dB nadie oye nada) salta libre. La
/// bajada puede ser más rápida (parar siempre es seguro). Es la regla de "nada de
/// sobresaltos a las 3 a.m." hecha código.
#[derive(Debug, Clone)]
pub struct GananciaSuave {
    actual_db: f32,
    lineal: f32,
    subida_db_por_muestra: f32,
    bajada_db_por_muestra: f32,
}

impl GananciaSuave {
    pub const SUBIDA_MAX_DB_S: f32 = 6.0;
    pub const BAJADA_MAX_DB_S: f32 = 24.0;
    pub const SILENCIO_DB: f32 = -90.0;
    pub const UMBRAL_AUDIBLE_DB: f32 = -50.0;

    pub fn nueva(sr: f32) -> Self {
        GananciaSuave {
            actual_db: Self::SILENCIO_DB,
            lineal: 0.0,
            subida_db_por_muestra: Self::SUBIDA_MAX_DB_S / sr,
            bajada_db_por_muestra: Self::BAJADA_MAX_DB_S / sr,
        }
    }
    /// Acerca la ganancia al objetivo (en dB) respetando las pendientes. Devuelve lineal.
    #[inline]
    pub fn hacia(&mut self, objetivo_db: f32) -> f32 {
        let objetivo = objetivo_db.max(Self::SILENCIO_DB);
        if objetivo == self.actual_db {
            return self.lineal;
        }
        if objetivo > self.actual_db {
            if self.actual_db < Self::UMBRAL_AUDIBLE_DB {
                self.actual_db = objetivo.min(Self::UMBRAL_AUDIBLE_DB);
            }
            self.actual_db = (self.actual_db + self.subida_db_por_muestra).min(objetivo);
        } else {
            self.actual_db = (self.actual_db - self.bajada_db_por_muestra).max(objetivo);
        }
        self.lineal = if self.actual_db <= Self::SILENCIO_DB + 0.01 { 0.0 } else { db_a_lin(self.actual_db) };
        self.lineal
    }
}

/// Curva de medio coseno: 0 → 1 suave.
#[inline]
pub fn suave(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    0.5 - 0.5 * (PI * x).cos()
}

/// Limitador de pico: ataque instantáneo, relajación exponencial. Garantiza |y| ≤ techo
/// y, como las recetas se mezclan por debajo del techo, casi nunca actúa.
#[derive(Debug, Clone)]
pub struct Limitador {
    techo: f32,
    reduccion: f32,
    relajacion: f32,
}

impl Limitador {
    pub fn nuevo(techo_db: f32, sr: f32) -> Self {
        Limitador { techo: db_a_lin(techo_db), reduccion: 1.0, relajacion: (-1.0 / (0.25 * sr)).exp() }
    }
    #[inline]
    pub fn procesar(&mut self, l: f32, r: f32) -> (f32, f32) {
        let pico = l.abs().max(r.abs());
        let necesaria = if pico > self.techo { self.techo / pico } else { 1.0 };
        if necesaria < self.reduccion {
            self.reduccion = necesaria;
        } else {
            self.reduccion = 1.0 - (1.0 - self.reduccion) * self.relajacion;
        }
        let g = self.reduccion;
        ((l * g).clamp(-self.techo, self.techo), (r * g).clamp(-self.techo, self.techo))
    }
}

/// FFT radix-2 en sitio (longitud potencia de dos). Para el espectrómetro, no para audio.
pub fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two() && im.len() == n);
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut largo = 2;
    while largo <= n {
        let ang = -2.0 * PI / largo as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        let mut i = 0;
        while i < n {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..largo / 2 {
                let (a, b) = (i + k, i + k + largo / 2);
                let tr = re[b] * cr - im[b] * ci;
                let ti = re[b] * ci + im[b] * cr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = ncr;
            }
            i += largo;
        }
        largo <<= 1;
    }
}
