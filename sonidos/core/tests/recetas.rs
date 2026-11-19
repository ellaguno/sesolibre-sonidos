//! Las recetas del repositorio: válidas, deterministas, bajo techo y sin sobresaltos.

use std::path::PathBuf;

use sonidos_core::dsp::{GananciaSuave, Limitador};
use sonidos_core::motor::medir;
use sonidos_core::{Banco, Motor, Receta, SR};

fn recetas() -> Vec<Receta> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../recetas");
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    v.sort();
    v.into_iter()
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .map(|p| Receta::desde_json(&std::fs::read_to_string(&p).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", p.display())))
        .collect()
}

fn motor(r: &Receta, semilla: Option<u64>) -> Motor {
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/audio");
    let banco = Banco::para_receta(r, &audio).unwrap_or_else(|e| panic!("{}: {e}", r.id));
    Motor::con_banco(r, semilla, &banco).unwrap_or_else(|e| panic!("{}: {e}", r.id))
}

#[test]
fn hay_recetas_y_todas_validan() {
    assert!(recetas().len() >= 8);
}

#[test]
fn las_muestras_del_indice_existen() {
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/audio");
    let i = sonidos_core::banco::indice();
    for a in i.instrumentos.values().flat_map(|d| d.notas.values()).chain(i.ambientes.values()) {
        assert!(audio.join(a).is_file(), "falta {a}");
    }
}

#[test]
fn una_receta_con_muestras_no_suena_sin_banco() {
    let r = recetas().into_iter().find(|r| r.id == "calmar.arpa-agua").unwrap();
    assert!(Motor::nuevo(&r, None).is_err());
    assert!(!sonidos_core::banco::archivos_necesarios(&r).unwrap().is_empty());
}

#[test]
fn receta_y_semilla_dan_el_mismo_sonido() {
    for r in recetas() {
        let a = motor(&r, None).rendir(Some(5.0));
        let b = motor(&r, None).rendir(Some(5.0));
        assert_eq!(a, b, "{} no es determinista", r.id);
        let c = motor(&r, Some(r.semilla + 1)).rendir(Some(5.0));
        if r.capas.iter().any(|c| matches!(c.gen, sonidos_core::Generador::Ruido { .. } | sonidos_core::Generador::Colchon { .. })) {
            assert_ne!(a, c, "{} ignora la semilla", r.id);
        }
    }
}

#[test]
fn nunca_supera_el_techo_ni_tiene_nan_ni_sobresaltos() {
    for r in recetas() {
        let m = medir(&motor(&r, None).rendir(Some(40.0)), SR);
        assert!(!m.tiene_nan, "{}: NaN", r.id);
        assert!(m.pico_db <= r.techo_db + 0.1, "{}: pico {} dB sobre techo {}", r.id, m.pico_db, r.techo_db);
        assert!(m.rms_db > -45.0, "{}: casi inaudible ({} dB RMS)", r.id, m.rms_db);
        assert!(m.salto_max_db < 6.0, "{}: salto de {} dB en 100 ms", r.id, m.salto_max_db);
    }
}

#[test]
fn termina_en_silencio() {
    for r in recetas() {
        let r = r.con_duracion(30.0);
        let mut m = motor(&r, None);
        let out = m.rendir(None);
        let cola = &out[out.len() - 200..];
        assert!(cola.iter().all(|x| x.abs() < 1e-3), "{} no acaba en silencio", r.id);
        assert!(m.terminado());
    }
}

#[test]
fn pausa_sin_clic_y_reanuda() {
    let r = recetas().into_iter().find(|r| r.id == "dormir.mar-cuerdas").unwrap();
    let mut m = motor(&r, None);
    let mut buf = vec![0.0f32; 4800 * 2];
    for _ in 0..200 {
        m.procesar(&mut buf); // 20 s: ya suena
    }
    assert!(buf.iter().any(|x| x.abs() > 1e-3));
    m.atenuar(true);
    let mut bloques = 0;
    while !m.en_silencio() {
        m.procesar(&mut buf);
        bloques += 1;
        assert!(bloques < 20, "la pausa tarda más de 2 s");
    }
    m.procesar(&mut buf);
    assert!(buf.iter().all(|x| *x == 0.0), "en pausa debe haber silencio exacto");
    m.atenuar(false);
    for _ in 0..30 {
        m.procesar(&mut buf);
    }
    assert!(buf.iter().any(|x| x.abs() > 1e-3), "no reanuda");
}

#[test]
fn extender_alarga_y_el_volumen_solo_baja() {
    let r = recetas().remove(0).con_duracion(60.0);
    let mut m = motor(&r, None);
    assert_eq!(m.duracion_s().round(), 60.0);
    assert_eq!(m.extender(900.0), 900.0);
    assert_eq!(m.duracion_s().round(), 960.0);
    assert!(m.extender(1e9) < 12.0 * 3600.0);
    assert!(m.duracion_s() <= 12.0 * 3600.0 + 1.0);

    let r = recetas().into_iter().find(|r| r.id == "dormir.mar-cuerdas").unwrap();
    let alto = medir(&motor(&r, None).rendir(Some(30.0)), SR);
    let mut m = motor(&r, None);
    m.volumen(6.0); // se ignora: nunca por encima de la receta
    let igual = medir(&m.rendir(Some(30.0)), SR);
    assert!((alto.rms_db - igual.rms_db).abs() < 0.1);
    let mut m = motor(&r, None);
    m.volumen(-12.0);
    let bajo = medir(&m.rendir(Some(30.0)), SR);
    assert!(bajo.rms_db < alto.rms_db - 8.0, "el volumen no baja: {} vs {}", bajo.rms_db, alto.rms_db);
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
    let base = recetas().into_iter().find(|r| r.id == "dormir.rosa-colchon").unwrap();
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

