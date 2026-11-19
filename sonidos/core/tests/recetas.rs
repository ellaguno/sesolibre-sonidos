//! Las recetas del repositorio: válidas, deterministas, bajo techo y sin sobresaltos.

use std::path::PathBuf;

use sonidos_core::dsp::{GananciaSuave, Limitador};
use sonidos_core::motor::medir;
use sonidos_core::{Motor, Receta, SR};

fn recetas() -> Vec<Receta> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../recetas");
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    v.sort();
    v.into_iter()
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .map(|p| Receta::desde_json(&std::fs::read_to_string(&p).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", p.display())))
        .collect()
}

#[test]
fn hay_recetas_y_todas_validan() {
    assert!(recetas().len() >= 6);
}

#[test]
fn receta_y_semilla_dan_el_mismo_sonido() {
    for r in recetas() {
        let a = Motor::nuevo(&r, None).unwrap().rendir(Some(5.0));
        let b = Motor::nuevo(&r, None).unwrap().rendir(Some(5.0));
        assert_eq!(a, b, "{} no es determinista", r.id);
        let c = Motor::nuevo(&r, Some(r.semilla + 1)).unwrap().rendir(Some(5.0));
        if r.capas.iter().any(|c| matches!(c.gen, sonidos_core::Generador::Ruido { .. } | sonidos_core::Generador::Colchon { .. })) {
            assert_ne!(a, c, "{} ignora la semilla", r.id);
        }
    }
}

#[test]
fn nunca_supera_el_techo_ni_tiene_nan_ni_sobresaltos() {
    for r in recetas() {
        let m = medir(&Motor::nuevo(&r, None).unwrap().rendir(Some(40.0)), SR);
        assert!(!m.tiene_nan, "{}: NaN", r.id);
        assert!(m.pico_db <= r.techo_db + 0.1, "{}: pico {} dB sobre techo {}", r.id, m.pico_db, r.techo_db);
        assert!(m.rms_db > -45.0, "{}: casi inaudible ({} dB RMS)", r.id, m.rms_db);
        assert!(m.salto_max_db < 6.0, "{}: salto de {} dB en 100 ms", r.id, m.salto_max_db);
    }
}

#[test]
fn termina_en_silencio() {
    let mut r = recetas().remove(0);
    r.duracion_s = 30.0;
    r.fade_out_s = 5.0;
    let out = Motor::nuevo(&r, None).unwrap().rendir(None);
    let cola = &out[out.len() - 200..];
    assert!(cola.iter().all(|x| x.abs() < 1e-3), "la receta no acaba en silencio");
    let mut m = Motor::nuevo(&r, None).unwrap();
    m.rendir(None);
    assert!(m.terminado());
}

#[test]
fn la_ganancia_no_sube_de_golpe() {
    let sr = 48_000.0;
    let mut g = GananciaSuave::nueva(sr);
    let mut ultimo = 0.0;
    for _ in 0..48_000 {
        ultimo = g.hacia(0.0);
    }
    // Salta libre hasta el umbral audible y de ahí sube como mucho SUBIDA_MAX_DB_S en un segundo.
    assert!(ultimo < 10f32.powf((GananciaSuave::UMBRAL_AUDIBLE_DB + GananciaSuave::SUBIDA_MAX_DB_S + 0.1) / 20.0));
    assert!(ultimo > 10f32.powf((GananciaSuave::UMBRAL_AUDIBLE_DB + GananciaSuave::SUBIDA_MAX_DB_S - 0.5) / 20.0));
}

#[test]
fn el_limitador_respeta_el_techo() {
    let mut l = Limitador::nuevo(-14.0, 48_000.0);
    let techo = 10f32.powf(-14.0 / 20.0);
    for i in 0..10_000 {
        let x = ((i as f32) * 0.01).sin() * 3.0;
        let (a, b) = l.procesar(x, -x);
        assert!(a.abs() <= techo + 1e-6 && b.abs() <= techo + 1e-6);
    }
}

#[test]
fn rechaza_promesas_y_descuidos() {
    let base = recetas().remove(0);
    let mut r = base.clone();
    r.ciencia.es = "Esto cura el insomnio".into();
    assert!(r.validar().is_err(), "debe rechazar 'cura'");
    let mut r = base.clone();
    r.nombre.en = "Healing tones".into();
    assert!(r.validar().is_err(), "debe rechazar 'healing'");
    let mut r = base.clone();
    r.techo_db = -3.0;
    assert!(r.validar().is_err(), "techo demasiado alto");
    let mut r = base.clone();
    r.advertencias.clear();
    assert!(r.validar().is_err(), "binaural sin advertencia de auriculares");
    let mut r = base.clone();
    r.capas[0].nivel_db = 3.0;
    assert!(r.validar().is_err(), "capa por encima de 0 dB");
}

