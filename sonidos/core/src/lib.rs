//! SesoLibre Sonidos — núcleo.
//!
//! Entrada: una **receta** (JSON) y una **semilla**. Salida: bloques de audio estéreo.
//! Sin interfaz, sin plataforma, sin hilos. Determinista: receta + semilla → el mismo
//! sonido, siempre. La app, Somnia y Mind lo consumen vía ligaduras; el CLI lo rinde a WAV.

pub mod banco;
pub mod dsp;
pub mod espacio;
pub mod gen;
pub mod mezcla;
pub mod motor;
pub mod receta;
pub mod rng;
pub mod teoria;
#[cfg(feature = "wav")]
pub mod render;

pub use banco::Banco;
pub use motor::{Medidas, Motor};
pub use receta::{Capa, Generador, Receta, Sala};

/// Frecuencia de muestreo de todo el núcleo.
pub const SR: u32 = 48_000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("receta inválida: {0}")]
    Receta(String),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "wav")]
    #[error("WAV: {0}")]
    Wav(#[from] hound::Error),
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),
}
