//! El espacio: una reverberación compartida a la que cada capa envía lo que quiera.
//! Red de 8 retardos con realimentación de Householder y amortiguación de agudos.

use crate::dsp::UnPolo;

const N: usize = 8;
/// Longitudes primas entre 30 y 65 ms a 48 kHz: sin modos que coincidan.
const RETARDOS: [usize; N] = [1423, 1597, 1787, 1999, 2239, 2503, 2797, 3121];

pub struct Reverb {
    lineas: Vec<Vec<f32>>,
    pos: [usize; N],
    ganancia: [f32; N],
    amortiguacion: Vec<UnPolo>,
}

impl Reverb {
    /// `rt60_s`: segundos que tarda la cola en caer 60 dB. `amortiguacion_hz`: por encima
    /// de aquí la cola se apaga antes (una sala, no un tanque de metal).
    pub fn nueva(rt60_s: f32, amortiguacion_hz: f32, sr: f32) -> Reverb {
        let escala = sr / 48_000.0;
        let largos: Vec<usize> = RETARDOS.iter().map(|&d| ((d as f32 * escala) as usize).max(8)).collect();
        let mut ganancia = [0.0; N];
        for (g, &d) in ganancia.iter_mut().zip(&largos) {
            *g = 10f32.powf(-3.0 * d as f32 / (rt60_s.max(0.2) * sr));
        }
        Reverb {
            lineas: largos.iter().map(|&d| vec![0.0; d]).collect(),
            pos: [0; N],
            ganancia,
            amortiguacion: (0..N).map(|_| UnPolo::paso_bajo(amortiguacion_hz, sr)).collect(),
        }
    }

    #[inline]
    pub fn procesar(&mut self, l: f32, r: f32) -> (f32, f32) {
        let mut salidas = [0.0f32; N];
        let mut suma = 0.0;
        for i in 0..N {
            salidas[i] = self.lineas[i][self.pos[i]];
            suma += salidas[i];
        }
        let reflejo = suma * (2.0 / N as f32);
        let (mut ol, mut or) = (0.0, 0.0);
        for i in 0..N {
            let entrada = if i % 2 == 0 { l } else { r };
            let x = self.amortiguacion[i].procesar((salidas[i] - reflejo) * self.ganancia[i]) + entrada;
            self.lineas[i][self.pos[i]] = x;
            self.pos[i] += 1;
            if self.pos[i] >= self.lineas[i].len() {
                self.pos[i] = 0;
            }
            if i % 2 == 0 {
                ol += salidas[i];
            } else {
                or += salidas[i];
            }
        }
        (ol * 0.25, or * 0.25)
    }
}
