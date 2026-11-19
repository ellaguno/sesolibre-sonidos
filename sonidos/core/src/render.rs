//! Render fuera de línea a WAV (16 bits, 48 kHz, estéreo).

use std::path::Path;

use crate::motor::{medir, Medidas, Motor};
use crate::receta::Receta;
use crate::{Error, SR};

pub fn rendir_wav(receta: &Receta, semilla: Option<u64>, segundos: Option<f32>, salida: &Path) -> Result<Medidas, Error> {
    let mut motor = Motor::nuevo(receta, semilla)?;
    let muestras = motor.rendir(segundos);
    let spec = hound::WavSpec { channels: 2, sample_rate: SR, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    let mut w = hound::WavWriter::create(salida, spec)?;
    for &x in &muestras {
        w.write_sample((x.clamp(-1.0, 1.0) * 32767.0) as i16)?;
    }
    w.finalize()?;
    Ok(medir(&muestras, SR))
}
