use super::{Cuadro, Fuente};
use crate::dsp::UnPolo;
use crate::receta::ColorRuido;
use crate::rng::Rng;

/// Ajustada midiendo: deja el marrón en RMS ≈ 0,25 como los otros colores.
const MARRON_GANANCIA: f32 = 2.5;

/// Ruido blanco, rosa (Paul Kellet) o marrón (integrador con fuga), decorrelado por canal
/// para que llene la cabeza sin "estar en el centro". Nominal: RMS ≈ 0,25.
pub struct Ruido {
    color: ColorRuido,
    rng: Rng,
    rosa: [[f32; 7]; 2],
    marron: [f32; 2],
    hp_marron: [UnPolo; 2],
    lp: Option<[UnPolo; 4]>,
    hp: Option<[UnPolo; 2]>,
}

impl Ruido {
    pub fn nuevo(color: ColorRuido, lp_hz: Option<f32>, hp_hz: Option<f32>, sr: f32, rng: Rng) -> Self {
        Ruido {
            color,
            rng,
            rosa: [[0.0; 7]; 2],
            marron: [0.0; 2],
            hp_marron: [UnPolo::paso_alto(30.0, sr), UnPolo::paso_alto(30.0, sr)],
            lp: lp_hz.map(|f| [UnPolo::paso_bajo(f, sr), UnPolo::paso_bajo(f, sr), UnPolo::paso_bajo(f, sr), UnPolo::paso_bajo(f, sr)]),
            hp: hp_hz.map(|f| [UnPolo::paso_alto(f, sr), UnPolo::paso_alto(f, sr)]),
        }
    }

    #[inline]
    fn muestra(&mut self, canal: usize) -> f32 {
        let blanco = self.rng.bipolar();
        match self.color {
            ColorRuido::Blanco => blanco * 0.43,
            ColorRuido::Rosa => {
                let b = &mut self.rosa[canal];
                b[0] = 0.99886 * b[0] + blanco * 0.0555179;
                b[1] = 0.99332 * b[1] + blanco * 0.0750759;
                b[2] = 0.96900 * b[2] + blanco * 0.1538520;
                b[3] = 0.86650 * b[3] + blanco * 0.3104856;
                b[4] = 0.55000 * b[4] + blanco * 0.5329522;
                b[5] = -0.7616 * b[5] - blanco * 0.0168980;
                let y = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + blanco * 0.5362;
                b[6] = blanco * 0.115926;
                y * 0.19
            }
            ColorRuido::Marron => {
                // Integrador con fuga (-6 dB/oct) y paso alto a 30 Hz: lo que no se oye no
                // gasta margen ni mueve el altavoz.
                let m = &mut self.marron[canal];
                *m = 0.999 * *m + blanco * 0.02;
                let y = self.hp_marron[canal].procesar(*m);
                y * MARRON_GANANCIA
            }
        }
    }
}

impl Fuente for Ruido {
    fn procesar(&mut self, _t0: f64, out: &mut [Cuadro]) {
        for c in out.iter_mut() {
            for canal in 0..2 {
                let mut y = self.muestra(canal);
                if let Some(lp) = &mut self.lp {
                    y = lp[canal * 2].procesar(y);
                    y = lp[canal * 2 + 1].procesar(y);
                }
                if let Some(hp) = &mut self.hp {
                    y = hp[canal].procesar(y);
                }
                c[canal] = y;
            }
        }
    }
}
