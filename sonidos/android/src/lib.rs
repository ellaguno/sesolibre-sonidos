//! El núcleo detrás de JNI. La clase Kotlin es `com.sesolibre.sonidos.motor.Nativo`.
//!
//! Dos manejadores opacos (punteros como `long`):
//! - **banco**: se crea, se le agregan los Ogg que la receta necesita y se entrega a `crear`;
//! - **sesión**: una receta sonando. `procesar` la llama el hilo de audio; lo demás
//!   (pausar, volumen, alargar) llega de otros hilos, por eso va tras un `Mutex`.
//!
//! Los errores de receta salen como `IllegalArgumentException` con el mensaje del núcleo.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use jni::objects::{JByteArray, JClass, JFloatArray, JString};
use jni::sys::{jboolean, jdouble, jfloat, jint, jlong, jstring};
use jni::JNIEnv;
use sonidos_core::{banco, Banco, Motor, Receta};

struct Sesion {
    motor: Mutex<Motor>,
    bloque: Mutex<Vec<f32>>,
}

/// `procesar` devuelve estos bits.
const SUENA: jint = 0;
const TERMINADO: jint = 1;
const EN_SILENCIO: jint = 2;
const ERROR: jint = -1;

fn lanzar(env: &mut JNIEnv, mensaje: &str) {
    let _ = env.throw_new("java/lang/IllegalArgumentException", mensaje);
}

fn texto(env: &mut JNIEnv, s: &JString) -> Option<String> {
    match env.get_string(s) {
        Ok(j) => Some(j.into()),
        Err(_) => {
            lanzar(env, "cadena inválida");
            None
        }
    }
}

fn receta_de(env: &mut JNIEnv, json: &JString) -> Option<Receta> {
    let s = texto(env, json)?;
    match Receta::desde_json(&s) {
        Ok(r) => Some(r),
        Err(e) => {
            lanzar(env, &e.to_string());
            None
        }
    }
}

fn nulo() -> jstring {
    std::ptr::null_mut()
}

/// `null` si la receta es válida; si no, el motivo.
#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_validar<'l>(mut env: JNIEnv<'l>, _c: JClass<'l>, receta: JString<'l>) -> jstring {
    let Some(s) = texto(&mut env, &receta) else { return nulo() };
    match Receta::desde_json(&s) {
        Ok(_) => nulo(),
        Err(e) => env.new_string(e.to_string()).map(|j| j.into_raw()).unwrap_or(nulo()),
    }
}

/// Los archivos de `assets/audio/` que la receta necesita, uno por línea.
#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_archivos<'l>(mut env: JNIEnv<'l>, _c: JClass<'l>, receta: JString<'l>) -> jstring {
    let Some(r) = receta_de(&mut env, &receta) else { return nulo() };
    match banco::archivos_necesarios(&r) {
        Ok(v) => env.new_string(v.join("\n")).map(|j| j.into_raw()).unwrap_or(nulo()),
        Err(e) => {
            lanzar(&mut env, &e.to_string());
            nulo()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_bancoNuevo(_env: JNIEnv, _c: JClass) -> jlong {
    Box::into_raw(Box::new(Banco::vacio())) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_bancoAgregar<'l>(
    mut env: JNIEnv<'l>,
    _c: JClass<'l>,
    banco: jlong,
    nombre: JString<'l>,
    ogg: JByteArray<'l>,
) {
    if banco == 0 {
        return lanzar(&mut env, "banco nulo");
    }
    let Some(nombre) = texto(&mut env, &nombre) else { return };
    let bytes = match env.convert_byte_array(&ogg) {
        Ok(b) => b,
        Err(_) => return lanzar(&mut env, "bytes inválidos"),
    };
    // SAFETY: `banco` viene de `bancoNuevo` y Kotlin no lo comparte entre hilos.
    let b = unsafe { &mut *(banco as *mut Banco) };
    if let Err(e) = b.agregar_ogg(&nombre, &bytes) {
        lanzar(&mut env, &e.to_string());
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_bancoLiberar(_env: JNIEnv, _c: JClass, banco: jlong) {
    if banco != 0 {
        // SAFETY: `banco` viene de `bancoNuevo` y no se vuelve a usar.
        drop(unsafe { Box::from_raw(banco as *mut Banco) });
    }
}

/// Crea la sesión. `duracion_s` ≤ 0 = la de la receta. `semilla` < 0 = la de la receta.
/// El banco se sigue pudiendo usar (y hay que liberarlo) después.
#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_crear<'l>(
    mut env: JNIEnv<'l>,
    _c: JClass<'l>,
    receta: JString<'l>,
    duracion_s: jfloat,
    semilla: jlong,
    banco: jlong,
) -> jlong {
    let Some(mut r) = receta_de(&mut env, &receta) else { return 0 };
    if duracion_s > 0.0 {
        r = r.con_duracion(duracion_s);
    }
    let vacio = Banco::vacio();
    // SAFETY: `banco` viene de `bancoNuevo`; solo se lee.
    let b = if banco == 0 { &vacio } else { unsafe { &*(banco as *const Banco) } };
    let semilla = if semilla < 0 { None } else { Some(semilla as u64) };
    match Motor::con_banco(&r, semilla, b) {
        Ok(m) => Box::into_raw(Box::new(Sesion { motor: Mutex::new(m), bloque: Mutex::new(Vec::new()) })) as jlong,
        Err(e) => {
            lanzar(&mut env, &e.to_string());
            0
        }
    }
}

fn sesion<'a>(h: jlong) -> Option<&'a Sesion> {
    if h == 0 {
        None
    } else {
        // SAFETY: `h` viene de `crear` y Kotlin no lo usa después de `liberar`.
        Some(unsafe { &*(h as *const Sesion) })
    }
}

/// Llena `salida` con `cuadros` cuadros estéreo intercalados. Devuelve SUENA (0),
/// TERMINADO (1), EN_SILENCIO (2: pausa completada) o -1 si algo falló.
#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_procesar<'l>(
    env: JNIEnv<'l>,
    _c: JClass<'l>,
    h: jlong,
    salida: JFloatArray<'l>,
    cuadros: jint,
) -> jint {
    let Some(s) = sesion(h) else { return ERROR };
    let n = (cuadros.max(0) as usize) * 2;
    let r = catch_unwind(AssertUnwindSafe(|| {
        let mut bloque = s.bloque.lock().unwrap_or_else(|e| e.into_inner());
        bloque.resize(n, 0.0);
        let mut m = s.motor.lock().unwrap_or_else(|e| e.into_inner());
        m.procesar(&mut bloque);
        let estado = if m.terminado() { TERMINADO } else { SUENA } | if m.en_silencio() { EN_SILENCIO } else { SUENA };
        drop(m);
        if env.set_float_array_region(&salida, 0, &bloque).is_err() {
            return ERROR;
        }
        estado
    }));
    r.unwrap_or(ERROR)
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_atenuar(_env: JNIEnv, _c: JClass, h: jlong, callar: jboolean) {
    if let Some(s) = sesion(h) {
        s.motor.lock().unwrap_or_else(|e| e.into_inner()).atenuar(callar != 0);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_volumen(_env: JNIEnv, _c: JClass, h: jlong, db: jfloat) {
    if let Some(s) = sesion(h) {
        s.motor.lock().unwrap_or_else(|e| e.into_inner()).volumen(db);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_extender(_env: JNIEnv, _c: JClass, h: jlong, segundos: jfloat) -> jfloat {
    sesion(h).map(|s| s.motor.lock().unwrap_or_else(|e| e.into_inner()).extender(segundos)).unwrap_or(0.0)
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_segundos(_env: JNIEnv, _c: JClass, h: jlong) -> jdouble {
    sesion(h).map(|s| s.motor.lock().unwrap_or_else(|e| e.into_inner()).segundos()).unwrap_or(0.0)
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_duracion(_env: JNIEnv, _c: JClass, h: jlong) -> jdouble {
    sesion(h).map(|s| s.motor.lock().unwrap_or_else(|e| e.into_inner()).duracion_s()).unwrap_or(0.0)
}

#[no_mangle]
pub extern "system" fn Java_com_sesolibre_sonidos_motor_Nativo_liberar(_env: JNIEnv, _c: JClass, h: jlong) {
    if h != 0 {
        // SAFETY: `h` viene de `crear` y no se vuelve a usar.
        drop(unsafe { Box::from_raw(h as *mut Sesion) });
    }
}
