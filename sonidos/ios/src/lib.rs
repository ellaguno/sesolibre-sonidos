//! El núcleo detrás de una ABI de C, para el plugin de iOS (Swift). La cabecera es
//! `include/sonidos.h`; el XCFramework lo arma `app/scripts/ios-nucleo.sh`.
//!
//! Las mismas operaciones que `sonidos/android` (JNI), con tres manejadores opacos:
//! - **banco**: se crea, se le agregan los Ogg que la receta necesita y se entrega a `crear`;
//! - **sesión**: una receta sonando. `procesar` la llama el hilo productor; lo demás
//!   (pausar, volumen, alargar) llega de otros hilos, por eso va tras un `Mutex`;
//! - **anillo**: un búfer circular estéreo de un productor y un consumidor, sin candados ni
//!   memoria nueva al leer. El hilo productor (Swift) escribe lo que sale de `procesar`; el
//!   hilo de render de Core Audio lo lee. Así el hilo de tiempo real nunca espera al núcleo.
//!
//! Las cadenas entran como UTF-8 terminadas en nulo. Las que salen las reserva Rust y se
//! liberan con `sonidos_texto_liberar`. Los errores salen por un `char **error` opcional.

// Es una ABI de C: los punteros los garantiza el llamador (Swift), como dice cada función.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::UnsafeCell;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use sonidos_core::{banco, Banco, Motor, Receta};

pub struct Sesion {
    motor: Mutex<Motor>,
}

/// `sonidos_procesar` devuelve estos bits.
pub const SONIDOS_SUENA: i32 = 0;
pub const SONIDOS_TERMINADO: i32 = 1;
pub const SONIDOS_EN_SILENCIO: i32 = 2;
pub const SONIDOS_ERROR: i32 = -1;

// ------------------------------------------------------------------ cadenas

fn a_c(s: &str) -> *mut c_char {
    // Un nulo dentro del mensaje cortaría la cadena: se cambia por un espacio.
    CString::new(s.replace('\0', " ")).map(CString::into_raw).unwrap_or(ptr::null_mut())
}

/// Escribe el error en `*error` (si el llamador lo pidió).
fn poner_error(error: *mut *mut c_char, mensaje: &str) {
    if !error.is_null() {
        // SAFETY: el llamador da un puntero válido a un `char *` (o nulo, ya descartado).
        unsafe { *error = a_c(mensaje) };
    }
}

fn texto<'a>(s: *const c_char) -> Result<&'a str, &'static str> {
    if s.is_null() {
        return Err("cadena nula");
    }
    // SAFETY: el llamador garantiza una cadena C válida mientras dura la llamada.
    unsafe { CStr::from_ptr(s) }.to_str().map_err(|_| "cadena inválida (no es UTF-8)")
}

fn receta_de(json: *const c_char) -> Result<Receta, String> {
    let s = texto(json).map_err(str::to_string)?;
    Receta::desde_json(s).map_err(|e| e.to_string())
}

/// Libera una cadena que entregó esta biblioteca. Acepta nulo.
#[no_mangle]
pub extern "C" fn sonidos_texto_liberar(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: `s` salió de `CString::into_raw` en esta biblioteca y no se vuelve a usar.
        drop(unsafe { CString::from_raw(s) });
    }
}

// ------------------------------------------------------------------ recetas

/// Nulo si la receta es válida; si no, el motivo (liberar con `sonidos_texto_liberar`).
#[no_mangle]
pub extern "C" fn sonidos_validar(receta: *const c_char) -> *mut c_char {
    match receta_de(receta) {
        Ok(_) => ptr::null_mut(),
        Err(e) => a_c(&e),
    }
}

/// Los archivos de `public/audio/` que la receta necesita, uno por línea (cadena vacía si
/// ninguno). Nulo si la receta no vale; el motivo queda en `*error`.
#[no_mangle]
pub extern "C" fn sonidos_archivos(receta: *const c_char, error: *mut *mut c_char) -> *mut c_char {
    let r = match receta_de(receta) {
        Ok(r) => r,
        Err(e) => {
            poner_error(error, &e);
            return ptr::null_mut();
        }
    };
    match banco::archivos_necesarios(&r) {
        Ok(v) => a_c(&v.join("\n")),
        Err(e) => {
            poner_error(error, &e.to_string());
            ptr::null_mut()
        }
    }
}

// ------------------------------------------------------------------ banco

#[no_mangle]
pub extern "C" fn sonidos_banco_nuevo() -> *mut Banco {
    Box::into_raw(Box::new(Banco::vacio()))
}

/// Decodifica un Ogg y lo agrega al banco con su nombre (la ruta relativa que dio
/// `sonidos_archivos`). Devuelve 0, o -1 con el motivo en `*error`.
#[no_mangle]
pub extern "C" fn sonidos_banco_agregar(
    banco: *mut Banco,
    nombre: *const c_char,
    ogg: *const u8,
    largo: usize,
    error: *mut *mut c_char,
) -> i32 {
    if banco.is_null() {
        poner_error(error, "banco nulo");
        return -1;
    }
    let nombre = match texto(nombre) {
        Ok(n) => n,
        Err(e) => {
            poner_error(error, e);
            return -1;
        }
    };
    if ogg.is_null() && largo > 0 {
        poner_error(error, "bytes nulos");
        return -1;
    }
    // SAFETY: `ogg` apunta a `largo` bytes válidos mientras dura la llamada.
    let bytes: &[u8] = if largo == 0 { &[] } else { unsafe { std::slice::from_raw_parts(ogg, largo) } };
    // SAFETY: `banco` viene de `sonidos_banco_nuevo` y Swift no lo comparte entre hilos.
    let b = unsafe { &mut *banco };
    match catch_unwind(AssertUnwindSafe(|| b.agregar_ogg(nombre, bytes))) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            poner_error(error, &e.to_string());
            -1
        }
        Err(_) => {
            poner_error(error, &format!("{nombre}: el decodificador falló"));
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn sonidos_banco_liberar(banco: *mut Banco) {
    if !banco.is_null() {
        // SAFETY: `banco` viene de `sonidos_banco_nuevo` y no se vuelve a usar.
        drop(unsafe { Box::from_raw(banco) });
    }
}

// ------------------------------------------------------------------ sesión

/// Crea la sesión. `duracion_s` ≤ 0 = la de la receta. `semilla` < 0 = la de la receta.
/// `banco` puede ser nulo (receta sin muestras); si no, se sigue pudiendo usar (y hay que
/// liberarlo) después. Nulo si falló, con el motivo en `*error`.
#[no_mangle]
pub extern "C" fn sonidos_crear(
    receta: *const c_char,
    duracion_s: f32,
    semilla: i64,
    banco: *const Banco,
    error: *mut *mut c_char,
) -> *mut Sesion {
    let mut r = match receta_de(receta) {
        Ok(r) => r,
        Err(e) => {
            poner_error(error, &e);
            return ptr::null_mut();
        }
    };
    if duracion_s > 0.0 {
        r = r.con_duracion(duracion_s);
    }
    let vacio = Banco::vacio();
    // SAFETY: `banco` viene de `sonidos_banco_nuevo`; solo se lee.
    let b = if banco.is_null() { &vacio } else { unsafe { &*banco } };
    let semilla = if semilla < 0 { None } else { Some(semilla as u64) };
    match catch_unwind(AssertUnwindSafe(|| Motor::con_banco(&r, semilla, b))) {
        Ok(Ok(m)) => Box::into_raw(Box::new(Sesion { motor: Mutex::new(m) })),
        Ok(Err(e)) => {
            poner_error(error, &e.to_string());
            ptr::null_mut()
        }
        Err(_) => {
            poner_error(error, "el núcleo falló al crear la sesión");
            ptr::null_mut()
        }
    }
}

fn sesion<'a>(h: *const Sesion) -> Option<&'a Sesion> {
    // SAFETY: `h` viene de `sonidos_crear` y Swift no lo usa después de `sonidos_liberar`.
    unsafe { h.as_ref() }
}

fn con_motor<T>(h: *const Sesion, por_defecto: T, f: impl FnOnce(&mut Motor) -> T) -> T {
    let Some(s) = sesion(h) else { return por_defecto };
    let mut m = s.motor.lock().unwrap_or_else(|e| e.into_inner());
    catch_unwind(AssertUnwindSafe(|| f(&mut m))).unwrap_or(por_defecto)
}

/// Llena `salida` con `cuadros` cuadros estéreo intercalados (2 × `cuadros` floats).
/// Devuelve bits: SONIDOS_TERMINADO (1), SONIDOS_EN_SILENCIO (2: pausa completada), o
/// SONIDOS_ERROR (-1) si algo falló. 0 = suena. No reserva memoria.
#[no_mangle]
pub extern "C" fn sonidos_procesar(h: *mut Sesion, salida: *mut f32, cuadros: u32) -> i32 {
    if salida.is_null() {
        return SONIDOS_ERROR;
    }
    // SAFETY: `salida` tiene espacio para `cuadros` × 2 floats.
    let out = unsafe { std::slice::from_raw_parts_mut(salida, cuadros as usize * 2) };
    con_motor(h, SONIDOS_ERROR, |m| {
        m.procesar(out);
        (if m.terminado() { SONIDOS_TERMINADO } else { SONIDOS_SUENA }) | (if m.en_silencio() { SONIDOS_EN_SILENCIO } else { SONIDOS_SUENA })
    })
}

/// Pausa (`callar` = true) o reanuda sin clic: hay que seguir procesando hasta EN_SILENCIO.
#[no_mangle]
pub extern "C" fn sonidos_atenuar(h: *mut Sesion, callar: bool) {
    con_motor(h, (), |m| m.atenuar(callar));
}

/// Volumen del usuario en dB (0 o menos).
#[no_mangle]
pub extern "C" fn sonidos_volumen(h: *mut Sesion, db: f32) {
    con_motor(h, (), |m| m.volumen(db));
}

/// "+15 min". Devuelve los segundos realmente añadidos.
#[no_mangle]
pub extern "C" fn sonidos_extender(h: *mut Sesion, segundos: f32) -> f32 {
    con_motor(h, 0.0, |m| m.extender(segundos))
}

#[no_mangle]
pub extern "C" fn sonidos_segundos(h: *const Sesion) -> f64 {
    con_motor(h, 0.0, |m| m.segundos())
}

#[no_mangle]
pub extern "C" fn sonidos_duracion(h: *const Sesion) -> f64 {
    con_motor(h, 0.0, |m| m.duracion_s())
}

/// Mueve el tono en vivo del laboratorio. Devuelve la frecuencia que quedó (20–2000 Hz).
#[no_mangle]
pub extern "C" fn sonidos_mando(h: *mut Sesion, hz: f32) -> f32 {
    con_motor(h, 0.0, |m| m.mando(hz))
}

/// Llena `bandas` (`n` valores, hasta 256) con el espectro de lo último que sonó: 0..1 por
/// banda, graves primero.
#[no_mangle]
pub extern "C" fn sonidos_espectro(h: *mut Sesion, bandas: *mut f32, n: u32) {
    if bandas.is_null() || n == 0 {
        return;
    }
    // SAFETY: `bandas` tiene espacio para `n` floats.
    let v = unsafe { std::slice::from_raw_parts_mut(bandas, (n as usize).min(256)) };
    v.iter_mut().for_each(|x| *x = 0.0);
    con_motor(h, (), |m| m.espectro(v));
}

#[no_mangle]
pub extern "C" fn sonidos_liberar(h: *mut Sesion) {
    if !h.is_null() {
        // SAFETY: `h` viene de `sonidos_crear` y no se vuelve a usar.
        drop(unsafe { Box::from_raw(h) });
    }
}

// ------------------------------------------------------------------ anillo

/// Búfer circular estéreo de un productor y un consumidor. `escrito` y `leido` cuentan
/// cuadros desde el principio (nunca dan la vuelta en la práctica: 2^64 cuadros).
pub struct Anillo {
    datos: Box<[UnsafeCell<f32>]>,
    cuadros: usize,
    escrito: AtomicUsize,
    leido: AtomicUsize,
}

// SAFETY: el productor solo toca la zona libre y el consumidor solo la ocupada; los
// contadores atómicos (Release/Acquire) ordenan los accesos.
unsafe impl Sync for Anillo {}
unsafe impl Send for Anillo {}

impl Anillo {
    pub fn nuevo(cuadros: usize) -> Anillo {
        let cuadros = cuadros.max(1);
        Anillo {
            datos: (0..cuadros * 2).map(|_| UnsafeCell::new(0.0)).collect(),
            cuadros,
            escrito: AtomicUsize::new(0),
            leido: AtomicUsize::new(0),
        }
    }

    /// Cuadros listos para leer.
    pub fn ocupado(&self) -> usize {
        self.escrito.load(Ordering::Acquire).wrapping_sub(self.leido.load(Ordering::Acquire))
    }

    /// Cuadros que caben.
    pub fn libre(&self) -> usize {
        self.cuadros - self.ocupado()
    }

    /// Productor: copia hasta `entrada.len() / 2` cuadros intercalados. Devuelve cuántos.
    pub fn escribir(&self, entrada: &[f32]) -> usize {
        let w = self.escrito.load(Ordering::Relaxed);
        let r = self.leido.load(Ordering::Acquire);
        let libre = self.cuadros - w.wrapping_sub(r);
        let n = (entrada.len() / 2).min(libre);
        for i in 0..n {
            let j = ((w + i) % self.cuadros) * 2;
            // SAFETY: la posición j está libre: el consumidor no la toca hasta que publiquemos.
            unsafe {
                *self.datos[j].get() = entrada[i * 2];
                *self.datos[j + 1].get() = entrada[i * 2 + 1];
            }
        }
        self.escrito.store(w.wrapping_add(n), Ordering::Release);
        n
    }

    /// Consumidor: llena `izq` y `der` (no intercalados, como los pide Core Audio). Lo que
    /// falte queda en silencio. Devuelve los cuadros que de verdad había.
    pub fn leer(&self, izq: &mut [f32], der: &mut [f32]) -> usize {
        let pedidos = izq.len().min(der.len());
        let r = self.leido.load(Ordering::Relaxed);
        let w = self.escrito.load(Ordering::Acquire);
        let n = w.wrapping_sub(r).min(pedidos);
        for i in 0..n {
            let j = ((r + i) % self.cuadros) * 2;
            // SAFETY: la posición j está ocupada: el productor no la toca hasta que la soltemos.
            unsafe {
                izq[i] = *self.datos[j].get();
                der[i] = *self.datos[j + 1].get();
            }
        }
        izq[n..pedidos].iter_mut().for_each(|x| *x = 0.0);
        der[n..pedidos].iter_mut().for_each(|x| *x = 0.0);
        self.leido.store(r.wrapping_add(n), Ordering::Release);
        n
    }
}

/// Un anillo de `cuadros` cuadros estéreo. Se reserva aquí y en ningún otro momento.
#[no_mangle]
pub extern "C" fn sonidos_anillo_nuevo(cuadros: u32) -> *mut Anillo {
    Box::into_raw(Box::new(Anillo::nuevo(cuadros as usize)))
}

fn anillo<'a>(a: *const Anillo) -> Option<&'a Anillo> {
    // SAFETY: `a` viene de `sonidos_anillo_nuevo` y no se usa después de liberarlo.
    unsafe { a.as_ref() }
}

/// Cuadros que caben ahora (productor).
#[no_mangle]
pub extern "C" fn sonidos_anillo_libre(a: *const Anillo) -> u32 {
    anillo(a).map(|a| a.libre() as u32).unwrap_or(0)
}

/// Cuadros que esperan a ser leídos.
#[no_mangle]
pub extern "C" fn sonidos_anillo_ocupado(a: *const Anillo) -> u32 {
    anillo(a).map(|a| a.ocupado() as u32).unwrap_or(0)
}

/// Productor: copia hasta `cuadros` cuadros intercalados de `entrada`. Devuelve cuántos cupieron.
#[no_mangle]
pub extern "C" fn sonidos_anillo_escribir(a: *const Anillo, entrada: *const f32, cuadros: u32) -> u32 {
    let Some(a) = anillo(a) else { return 0 };
    if entrada.is_null() {
        return 0;
    }
    // SAFETY: `entrada` tiene `cuadros` × 2 floats.
    let e = unsafe { std::slice::from_raw_parts(entrada, cuadros as usize * 2) };
    a.escribir(e) as u32
}

/// Consumidor (hilo de render): llena `izq` y `der` con `cuadros` cuadros, en silencio lo
/// que falte. Ni candados ni memoria nueva. Devuelve los cuadros que de verdad había.
#[no_mangle]
pub extern "C" fn sonidos_anillo_leer(a: *const Anillo, izq: *mut f32, der: *mut f32, cuadros: u32) -> u32 {
    if izq.is_null() || der.is_null() {
        return 0;
    }
    let n = cuadros as usize;
    // SAFETY: `izq` y `der` tienen `cuadros` floats cada uno y no se solapan.
    let (i, d) = unsafe { (std::slice::from_raw_parts_mut(izq, n), std::slice::from_raw_parts_mut(der, n)) };
    match anillo(a) {
        Some(a) => a.leer(i, d) as u32,
        None => {
            i.iter_mut().for_each(|x| *x = 0.0);
            d.iter_mut().for_each(|x| *x = 0.0);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn sonidos_anillo_liberar(a: *mut Anillo) {
    if !a.is_null() {
        // SAFETY: `a` viene de `sonidos_anillo_nuevo` y nadie lo vuelve a usar.
        drop(unsafe { Box::from_raw(a) });
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const GAMMA: &str = include_str!("../../../recetas/concentrar.gamma-40.json");

    fn c(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    fn tomar(p: *mut c_char) -> Option<String> {
        if p.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
        sonidos_texto_liberar(p);
        Some(s)
    }

    #[test]
    fn valida_y_rechaza() {
        assert_eq!(tomar(sonidos_validar(c(GAMMA).as_ptr())), None);
        assert!(tomar(sonidos_validar(c("{}").as_ptr())).is_some());
        assert!(tomar(sonidos_validar(ptr::null())).is_some());
    }

    #[test]
    fn archivos_y_errores() {
        let mut err: *mut c_char = ptr::null_mut();
        let a = tomar(sonidos_archivos(c(GAMMA).as_ptr(), &mut err));
        assert_eq!(a.as_deref(), Some(""));
        assert!(err.is_null());
        assert!(sonidos_archivos(c("no es json").as_ptr(), &mut err).is_null());
        assert!(tomar(err).is_some());
    }

    #[test]
    fn banco_rechaza_bytes_malos() {
        let b = sonidos_banco_nuevo();
        let mut err: *mut c_char = ptr::null_mut();
        let basura = [1u8, 2, 3, 4];
        assert_eq!(sonidos_banco_agregar(b, c("x.ogg").as_ptr(), basura.as_ptr(), basura.len(), &mut err), -1);
        assert!(tomar(err).unwrap().contains("x.ogg"));
        sonidos_banco_liberar(b);
    }

    #[test]
    fn sesion_suena_pausa_y_termina() {
        let mut err: *mut c_char = ptr::null_mut();
        let h = sonidos_crear(c(GAMMA).as_ptr(), 10.0, 7, ptr::null(), &mut err);
        assert!(!h.is_null(), "{:?}", tomar(err));
        assert!((sonidos_duracion(h) - 10.0).abs() < 1e-3);
        let mut bloque = vec![0.0f32; 1024 * 2];
        let mut e = sonidos_procesar(h, bloque.as_mut_ptr(), 1024);
        assert!(e >= 0 && e & SONIDOS_TERMINADO == 0);
        sonidos_volumen(h, -6.0);
        assert!(sonidos_extender(h, 2.0) > 0.0);
        assert!(sonidos_mando(h, 5.0) >= 20.0);
        let mut bandas = [0.0f32; 16];
        sonidos_espectro(h, bandas.as_mut_ptr(), 16);
        assert!(bandas.iter().all(|x| (0.0..=1.0).contains(x)));
        // Pausa: en algún momento calla.
        sonidos_atenuar(h, true);
        let mut vueltas = 0;
        while e & SONIDOS_EN_SILENCIO == 0 {
            e = sonidos_procesar(h, bloque.as_mut_ptr(), 1024);
            vueltas += 1;
            assert!(vueltas < 500);
        }
        // Reanuda y deja que termine (12 s en total).
        sonidos_atenuar(h, false);
        vueltas = 0;
        while e & SONIDOS_TERMINADO == 0 {
            e = sonidos_procesar(h, bloque.as_mut_ptr(), 1024);
            assert!(e >= 0);
            vueltas += 1;
            assert!(vueltas < 2000);
        }
        sonidos_liberar(h);
        // Manejadores nulos: no truenan.
        assert_eq!(sonidos_procesar(ptr::null_mut(), bloque.as_mut_ptr(), 1), SONIDOS_ERROR);
        assert_eq!(sonidos_segundos(ptr::null()), 0.0);
    }

    #[test]
    fn anillo_da_la_vuelta_y_rellena_con_silencio() {
        let a = sonidos_anillo_nuevo(4);
        assert_eq!(sonidos_anillo_libre(a), 4);
        let entrada = [1.0, -1.0, 2.0, -2.0, 3.0, -3.0];
        assert_eq!(sonidos_anillo_escribir(a, entrada.as_ptr(), 3), 3);
        assert_eq!(sonidos_anillo_escribir(a, entrada.as_ptr(), 3), 1);
        assert_eq!(sonidos_anillo_ocupado(a), 4);
        let (mut i, mut d) = ([9.0f32; 3], [9.0f32; 3]);
        assert_eq!(sonidos_anillo_leer(a, i.as_mut_ptr(), d.as_mut_ptr(), 3), 3);
        assert_eq!((i, d), ([1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]));
        assert_eq!(sonidos_anillo_escribir(a, entrada.as_ptr(), 3), 3);
        let (mut i, mut d) = ([9.0f32; 6], [9.0f32; 6]);
        assert_eq!(sonidos_anillo_leer(a, i.as_mut_ptr(), d.as_mut_ptr(), 6), 4);
        assert_eq!(i, [1.0, 1.0, 2.0, 3.0, 0.0, 0.0]);
        assert_eq!(d, [-1.0, -1.0, -2.0, -3.0, 0.0, 0.0]);
        sonidos_anillo_liberar(a);
    }

    #[test]
    fn anillo_entre_hilos() {
        let a = Anillo::nuevo(256);
        let total = 100_000usize;
        std::thread::scope(|s| {
            s.spawn(|| {
                let mut k = 0usize;
                while k < total {
                    let n = (total - k).min(64);
                    let v: Vec<f32> = (k..k + n).flat_map(|x| [x as f32, -(x as f32)]).collect();
                    k += a.escribir(&v);
                    std::thread::yield_now();
                }
            });
            let mut k = 0usize;
            let (mut i, mut d) = (vec![0.0f32; 100], vec![0.0f32; 100]);
            while k < total {
                let n = a.leer(&mut i, &mut d);
                for j in 0..n {
                    assert_eq!(i[j], (k + j) as f32);
                    assert_eq!(d[j], -((k + j) as f32));
                }
                k += n;
            }
        });
    }
}
