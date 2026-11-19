//! Generador determinista (SplitMix64). Nunca `rand` del sistema dentro del núcleo.

#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn nueva(semilla: u64) -> Self {
        Rng(semilla ^ 0x9E37_79B9_7F4A_7C15)
    }

    /// Deriva un generador independiente para una capa o voz.
    pub fn derivar(&self, etiqueta: u64) -> Rng {
        let mut r = Rng(self.0.wrapping_add(etiqueta.wrapping_mul(0xBF58_476D_1CE4_E5B9)));
        r.u64();
        r
    }

    pub fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniforme en [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniforme en [-1, 1).
    pub fn bipolar(&mut self) -> f32 {
        self.f32() * 2.0 - 1.0
    }

    /// Uniforme en [a, b).
    pub fn rango(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f32()
    }
}
