import Foundation
import SonidosNucleo

/// El núcleo de sonido (Rust, `sonidos/ios` → SonidosNucleo.xcframework) con nombres de
/// Swift. Los `OpaquePointer` son manejadores opacos: un banco de muestras, una sesión (una
/// receta sonando) y un anillo (el búfer entre el hilo productor y el de render). Los errores
/// de receta llegan como `Nativo.Fallo` con el mensaje del núcleo.
enum Nativo {
    /// Bits que devuelve `procesar`.
    static let terminado: Int32 = 1
    static let enSilencio: Int32 = 2

    struct Fallo: Error {
        let mensaje: String
    }

    /// Copia una cadena del núcleo y la libera.
    private static func tomar(_ p: UnsafeMutablePointer<CChar>?) -> String? {
        guard let p = p else { return nil }
        defer { sonidos_texto_liberar(p) }
        return String(cString: p)
    }

    /// `nil` si la receta es válida; si no, el motivo.
    static func validar(_ receta: String) -> String? {
        return tomar(sonidos_validar(receta))
    }

    /// Archivos de `public/audio/` que la receta necesita.
    static func archivos(_ receta: String) throws -> [String] {
        var error: UnsafeMutablePointer<CChar>? = nil
        guard let lista = tomar(sonidos_archivos(receta, &error)) else {
            throw Fallo(mensaje: tomar(error) ?? "receta inválida")
        }
        return lista.split(separator: "\n").map(String.init).filter { !$0.isEmpty }
    }

    static func bancoNuevo() -> OpaquePointer? {
        return sonidos_banco_nuevo()
    }

    static func bancoAgregar(_ banco: OpaquePointer, _ nombre: String, _ ogg: Data) throws {
        var error: UnsafeMutablePointer<CChar>? = nil
        let r: Int32 = ogg.withUnsafeBytes { (crudo: UnsafeRawBufferPointer) -> Int32 in
            let bytes = crudo.bindMemory(to: UInt8.self)
            return sonidos_banco_agregar(banco, nombre, bytes.baseAddress, bytes.count, &error)
        }
        if r != 0 {
            throw Fallo(mensaje: tomar(error) ?? "\(nombre): no se pudo decodificar")
        }
    }

    static func bancoLiberar(_ banco: OpaquePointer?) {
        sonidos_banco_liberar(banco)
    }

    /// `duracionS` ≤ 0 y `semilla` < 0 = las de la receta.
    static func crear(_ receta: String, _ duracionS: Float, _ semilla: Int64, _ banco: OpaquePointer?) throws -> OpaquePointer {
        var error: UnsafeMutablePointer<CChar>? = nil
        guard let s = sonidos_crear(receta, duracionS, semilla, banco, &error) else {
            throw Fallo(mensaje: tomar(error) ?? "no se pudo crear la sesión")
        }
        return s
    }

    /// Llena `salida` (estéreo intercalado, 2 × `cuadros` floats). Devuelve bits de estado,
    /// o -1 si falló.
    static func procesar(_ sesion: OpaquePointer, _ salida: UnsafeMutablePointer<Float>, _ cuadros: UInt32) -> Int32 {
        return sonidos_procesar(sesion, salida, cuadros)
    }

    static func atenuar(_ sesion: OpaquePointer, _ callar: Bool) {
        sonidos_atenuar(sesion, callar)
    }

    static func volumen(_ sesion: OpaquePointer, _ db: Float) {
        sonidos_volumen(sesion, db)
    }

    static func extender(_ sesion: OpaquePointer, _ segundos: Float) -> Float {
        return sonidos_extender(sesion, segundos)
    }

    static func segundos(_ sesion: OpaquePointer) -> Double {
        return sonidos_segundos(sesion)
    }

    static func duracion(_ sesion: OpaquePointer) -> Double {
        return sonidos_duracion(sesion)
    }

    /// Mueve el tono en vivo (capa `mando`). Devuelve la frecuencia que quedó (20–2000 Hz).
    static func mando(_ sesion: OpaquePointer, _ hz: Float) -> Float {
        return sonidos_mando(sesion, hz)
    }

    /// Espectro de lo último que sonó: 0..1 por banda, graves primero.
    static func espectro(_ sesion: OpaquePointer, _ n: Int) -> [Float] {
        var bandas = [Float](repeating: 0, count: n)
        bandas.withUnsafeMutableBufferPointer { b in
            sonidos_espectro(sesion, b.baseAddress, UInt32(b.count))
        }
        return bandas
    }

    static func liberar(_ sesion: OpaquePointer) {
        sonidos_liberar(sesion)
    }

    // ------------------------------------------------------------------ anillo

    static func anilloNuevo(_ cuadros: UInt32) -> OpaquePointer? {
        return sonidos_anillo_nuevo(cuadros)
    }

    static func anilloLibre(_ anillo: OpaquePointer) -> UInt32 {
        return sonidos_anillo_libre(anillo)
    }

    static func anilloOcupado(_ anillo: OpaquePointer) -> UInt32 {
        return sonidos_anillo_ocupado(anillo)
    }

    static func anilloEscribir(_ anillo: OpaquePointer, _ entrada: UnsafePointer<Float>, _ cuadros: UInt32) -> UInt32 {
        return sonidos_anillo_escribir(anillo, entrada, cuadros)
    }

    /// Para el hilo de render: ni candados ni memoria nueva.
    @inline(__always)
    static func anilloLeer(_ anillo: OpaquePointer, _ izq: UnsafeMutablePointer<Float>, _ der: UnsafeMutablePointer<Float>, _ cuadros: UInt32) -> UInt32 {
        return sonidos_anillo_leer(anillo, izq, der, cuadros)
    }

    static func anilloLiberar(_ anillo: OpaquePointer) {
        sonidos_anillo_liberar(anillo)
    }
}
