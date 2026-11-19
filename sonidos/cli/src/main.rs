//! `sonidos`: rinde recetas a WAV y las mide.
//!
//!   sonidos render <receta.json> <salida.wav> [--segundos S] [--semilla N]
//!   sonidos validar <receta.json>...
//!   sonidos lote <dir_recetas> <dir_salida> [--segundos S]
//!
//! Las recetas con muestras las leen de `--audio DIR` (por defecto `assets/audio`).

use std::path::{Path, PathBuf};
use std::process::exit;

use sonidos_core::render::rendir_wav;
use sonidos_core::{Banco, Medidas, Receta};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("render") => render(&args[1..]),
        Some("validar") => validar(&args[1..]),
        Some("lote") => lote(&args[1..]),
        _ => {
            eprintln!("uso:\n  sonidos render <receta.json> <salida.wav> [--segundos S] [--semilla N]\n  sonidos validar <receta.json>...\n  sonidos lote <dir_recetas> <dir_salida> [--segundos S]");
            exit(2);
        }
    };
    if let Err(e) = r {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn opcion<T: std::str::FromStr>(args: &[String], nombre: &str) -> Option<T> {
    args.iter().position(|a| a == nombre).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok())
}

fn banco(args: &[String], receta: &Receta) -> Result<Banco, Box<dyn std::error::Error>> {
    let dir: String = opcion(args, "--audio").unwrap_or_else(|| "assets/audio".to_string());
    Ok(Banco::para_receta(receta, Path::new(&dir))?)
}

fn cargar(p: &Path) -> Result<Receta, Box<dyn std::error::Error>> {
    Ok(Receta::desde_json(&std::fs::read_to_string(p)?)?)
}

fn render(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let entrada = PathBuf::from(args.first().ok_or("falta la receta")?);
    let salida = PathBuf::from(args.get(1).ok_or("falta la salida")?);
    let receta = cargar(&entrada)?;
    let inicio = std::time::Instant::now();
    let m = rendir_wav(&receta, &banco(args, &receta)?, opcion(args, "--semilla"), opcion(args, "--segundos"), &salida)?;
    eprintln!("{:.0}× tiempo real", m.segundos / inicio.elapsed().as_secs_f32());
    println!("{}", fila(&receta, &m, &salida));
    Ok(())
}

fn validar(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut mal = 0;
    for a in args {
        match cargar(Path::new(a)) {
            Ok(r) => println!("ok   {} ({:?}, {} capas, {:.0} s)", r.id, r.sala, r.capas.len(), r.duracion_s),
            Err(e) => {
                mal += 1;
                println!("MAL  {a}: {e}");
            }
        }
    }
    if mal > 0 {
        Err(format!("{mal} receta(s) inválida(s)").into())
    } else {
        Ok(())
    }
}

fn lote(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(args.first().ok_or("falta el directorio de recetas")?);
    let salida = PathBuf::from(args.get(1).ok_or("falta el directorio de salida")?);
    let segundos: Option<f32> = opcion(args, "--segundos");
    std::fs::create_dir_all(&salida)?;
    let mut entradas: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    entradas.sort();
    println!("{:<34} {:>7} {:>8} {:>8} {:>7}  archivo", "receta", "seg", "pico", "rms", "salto");
    for p in entradas {
        let receta = cargar(&p)?;
        let wav = salida.join(format!("{}.wav", receta.id));
        let m = rendir_wav(&receta, &banco(args, &receta)?, None, segundos, &wav)?;
        println!("{}", fila(&receta, &m, &wav));
    }
    Ok(())
}

fn fila(r: &Receta, m: &Medidas, p: &Path) -> String {
    format!(
        "{:<34} {:>6.1}s {:>6.1}dB {:>6.1}dB {:>5.1}dB  {}{}",
        r.id,
        m.segundos,
        m.pico_db,
        m.rms_db,
        m.salto_max_db,
        p.display(),
        if m.tiene_nan { "  ¡NaN!" } else { "" }
    )
}
