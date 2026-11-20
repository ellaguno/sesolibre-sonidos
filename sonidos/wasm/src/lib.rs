//! El núcleo en WebAssembly, sin wasm-bindgen: una ABI de C mínima que el AudioWorklet (y
//! cualquier JS) usa leyendo y escribiendo la memoria lineal.
//!
//! Convenciones:
//! - el texto y los bytes entran como (puntero, longitud) a memoria pedida con `reservar`;
//! - el texto sale por `texto_ptr()` / `texto_len()` (el último resultado o error);
//! - una función que falla devuelve 0 (o -1) y deja el motivo en ese texto;
//! - un solo hilo: todo el estado es global.

use std::cell::RefCell;

use sonidos_core::{banco, Banco, Motor, Receta};

thread_local! {
    static TEXTO: RefCell<String> = const { RefCell::new(String::new()) };
    static BANCO: RefCell<Banco> = RefCell::new(Banco::vacio());
    static MOTOR: RefCell<Option<Motor>> = const { RefCell::new(None) };
    static SALIDA: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

fn poner_texto(s: String) {
    TEXTO.with(|t| *t.borrow_mut() = s);
}

/// SAFETY: (ptr, len) viene de `reservar` y JS lo llenó antes de llamar.
unsafe fn bytes<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(ptr, len)
    }
}

fn receta_de(ptr: *const u8, len: usize) -> Option<Receta> {
    let json = match std::str::from_utf8(unsafe { bytes(ptr, len) }) {
        Ok(s) => s,
        Err(_) => {
            poner_texto("la receta no es UTF-8".into());
            return None;
        }
    };
    match Receta::desde_json(json) {
        Ok(r) => Some(r),
        Err(e) => {
            poner_texto(e.to_string());
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn reservar(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// # Safety
/// (ptr, len) debe venir de `reservar(len)`.
#[no_mangle]
pub unsafe extern "C" fn liberar(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Vec::from_raw_parts(ptr, 0, len.max(1)));
    }
}

#[no_mangle]
pub extern "C" fn texto_ptr() -> *const u8 {
    TEXTO.with(|t| t.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn texto_len() -> usize {
    TEXTO.with(|t| t.borrow().len())
}

/// 1 si la receta es válida; 0 si no (el motivo queda en el texto).
#[no_mangle]
pub extern "C" fn validar(ptr: *const u8, len: usize) -> i32 {
    receta_de(ptr, len).is_some() as i32
}

/// Deja en el texto los archivos de audio que la receta necesita, uno por línea. 1 = bien.
#[no_mangle]
pub extern "C" fn archivos(ptr: *const u8, len: usize) -> i32 {
    let Some(r) = receta_de(ptr, len) else { return 0 };
    match banco::archivos_necesarios(&r) {
        Ok(v) => {
            poner_texto(v.join("\n"));
            1
        }
        Err(e) => {
            poner_texto(e.to_string());
            0
        }
    }
}

/// Añade un Ogg al banco con ese nombre. 1 = bien.
#[no_mangle]
pub extern "C" fn banco_agregar(nombre: *const u8, nombre_len: usize, ogg: *const u8, ogg_len: usize) -> i32 {
    let nombre = String::from_utf8_lossy(unsafe { bytes(nombre, nombre_len) }).into_owned();
    match BANCO.with(|b| b.borrow_mut().agregar_ogg(&nombre, unsafe { bytes(ogg, ogg_len) })) {
        Ok(()) => 1,
        Err(e) => {
            poner_texto(e.to_string());
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn banco_vaciar() {
    BANCO.with(|b| *b.borrow_mut() = Banco::vacio());
}

/// Empieza una receta (sustituye a la anterior). `duracion_s` ≤ 0 = la de la receta. 1 = bien.
#[no_mangle]
pub extern "C" fn crear(ptr: *const u8, len: usize, duracion_s: f32) -> i32 {
    let Some(mut r) = receta_de(ptr, len) else { return 0 };
    if duracion_s > 0.0 {
        r = r.con_duracion(duracion_s);
    }
    match BANCO.with(|b| Motor::con_banco(&r, None, &b.borrow())) {
        Ok(m) => {
            MOTOR.with(|x| *x.borrow_mut() = Some(m));
            1
        }
        Err(e) => {
            poner_texto(e.to_string());
            0
        }
    }
}

/// Genera `cuadros` cuadros estéreo intercalados y devuelve dónde quedaron (válido hasta la
/// próxima llamada). Nulo si no hay receta.
#[no_mangle]
pub extern "C" fn procesar(cuadros: usize) -> *const f32 {
    MOTOR.with(|m| {
        let mut m = m.borrow_mut();
        let Some(motor) = m.as_mut() else { return std::ptr::null() };
        SALIDA.with(|s| {
            let mut s = s.borrow_mut();
            s.resize(cuadros * 2, 0.0);
            motor.procesar(&mut s);
            s.as_ptr()
        })
    })
}

/// Bits: 1 = terminado, 2 = en silencio (pausa completada). -1 = no hay receta.
#[no_mangle]
pub extern "C" fn estado() -> i32 {
    MOTOR.with(|m| m.borrow().as_ref().map(|m| m.terminado() as i32 | (m.en_silencio() as i32) << 1).unwrap_or(-1))
}

fn con_motor<T>(def: T, f: impl FnOnce(&mut Motor) -> T) -> T {
    MOTOR.with(|m| m.borrow_mut().as_mut().map(f).unwrap_or(def))
}

#[no_mangle]
pub extern "C" fn atenuar(callar: i32) {
    con_motor((), |m| m.atenuar(callar != 0));
}

#[no_mangle]
pub extern "C" fn volumen(db: f32) {
    con_motor((), |m| m.volumen(db));
}

#[no_mangle]
pub extern "C" fn extender(segundos: f32) -> f32 {
    con_motor(0.0, |m| m.extender(segundos))
}

#[no_mangle]
pub extern "C" fn segundos() -> f64 {
    con_motor(0.0, |m| m.segundos())
}

#[no_mangle]
pub extern "C" fn duracion() -> f64 {
    con_motor(0.0, |m| m.duracion_s())
}

#[no_mangle]
pub extern "C" fn mando(hz: f32) -> f32 {
    con_motor(0.0, |m| m.mando(hz))
}

/// Calcula el espectro en `bandas` bandas y devuelve dónde quedó (f32, 0..1).
#[no_mangle]
pub extern "C" fn espectro(bandas: usize) -> *const f32 {
    SALIDA.with(|s| {
        let mut s = s.borrow_mut();
        s.clear();
        s.resize(bandas.min(256), 0.0);
        con_motor((), |m| m.espectro(&mut s));
        s.as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn parar() {
    MOTOR.with(|m| *m.borrow_mut() = None);
}
