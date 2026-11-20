import Capacitor
import Foundation

/// La cara web del reproductor: la app (JS) elige la receta y este plugin se la pasa al
/// `SonidosPlayer`. Los métodos llegan en la cola del puente; todo lo que toca al reproductor
/// pasa al hilo principal. Emite el evento `estado` cada vez que algo cambia y `error` si una
/// receta no pudo sonar. Los mismos métodos y eventos que el plugin de Android.
@objc(SonidosPlugin)
public class SonidosPlugin: CAPPlugin, CAPBridgedPlugin {
    public let identifier = "SonidosPlugin"
    public let jsName = "Sonidos"
    public let pluginMethods: [CAPPluginMethod] = [
        CAPPluginMethod(name: "tocar", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "pausar", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "reanudar", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "parar", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "extender", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "volumen", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "espectro", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "mando", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "validar", returnType: CAPPluginReturnPromise),
        CAPPluginMethod(name: "estado", returnType: CAPPluginReturnPromise)
    ]

    /// Solo se toca en el hilo principal.
    private var reproductor: SonidosPlayer?

    override public func load() {
        enPrincipal { _ in }
    }

    /// Entrega el reproductor (lo crea la primera vez) en el hilo principal.
    private func enPrincipal(_ usar: @escaping (SonidosPlayer) -> Void) {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            if let p = self.reproductor {
                usar(p)
                return
            }
            let p = SonidosPlayer()
            p.alCambiar = { [weak self] in self?.avisar() }
            p.alError = { [weak self] mensaje in
                self?.notifyListeners("error", data: ["mensaje": mensaje])
            }
            self.reproductor = p
            usar(p)
        }
    }

    @objc func tocar(_ call: CAPPluginCall) {
        guard let receta = call.getString("receta") else {
            call.reject("falta la receta")
            return
        }
        if let motivo = Nativo.validar(receta) {
            call.reject(motivo)
            return
        }
        let minutos = call.getFloat("minutos") ?? 0
        let recetaId = call.getString("recetaId") ?? "receta"
        let titulo = call.getString("titulo") ?? "Sonidos"
        let subtitulo = call.getString("subtitulo") ?? ""
        let volumenDb = call.getFloat("volumenDb") ?? 0
        enPrincipal { p in
            p.tocar(receta: receta, recetaId: recetaId, titulo: titulo, subtitulo: subtitulo, duracionS: minutos * 60, volumenDb: volumenDb)
            call.resolve()
        }
    }

    @objc func pausar(_ call: CAPPluginCall) {
        enPrincipal { p in
            p.pausar()
            call.resolve()
        }
    }

    @objc func reanudar(_ call: CAPPluginCall) {
        enPrincipal { p in
            p.reanudar()
            call.resolve()
        }
    }

    @objc func parar(_ call: CAPPluginCall) {
        enPrincipal { p in
            p.parar()
            call.resolve()
        }
    }

    @objc func extender(_ call: CAPPluginCall) {
        guard let minutos = call.getFloat("minutos") else {
            call.reject("faltan los minutos")
            return
        }
        enPrincipal { p in
            let s = p.extender(minutos * 60)
            call.resolve(["minutos": Double(s) / 60])
        }
    }

    @objc func volumen(_ call: CAPPluginCall) {
        guard let db = call.getFloat("db") else {
            call.reject("faltan los dB")
            return
        }
        enPrincipal { p in
            p.volumen(db)
            call.resolve()
        }
    }

    /// El espectro de lo que suena: `bandas` valores de 0 a 1. Vacío si no suena nada.
    @objc func espectro(_ call: CAPPluginCall) {
        let n = min(max(call.getInt("bandas") ?? 48, 8), 128)
        enPrincipal { p in
            let v = p.espectro(n) ?? []
            call.resolve(["bandas": v.map { Double($0) }])
        }
    }

    /// Mueve el tono en vivo del laboratorio de cimática.
    @objc func mando(_ call: CAPPluginCall) {
        guard let hz = call.getFloat("hz") else {
            call.reject("faltan los Hz")
            return
        }
        enPrincipal { p in
            call.resolve(["hz": Double(p.mando(hz))])
        }
    }

    /// `motivo` nulo si el núcleo acepta la receta; si no, por qué no.
    @objc func validar(_ call: CAPPluginCall) {
        guard let receta = call.getString("receta") else {
            call.reject("falta la receta")
            return
        }
        let motivo: Any = Nativo.validar(receta) ?? NSNull()
        call.resolve(["motivo": motivo])
    }

    @objc func estado(_ call: CAPPluginCall) {
        enPrincipal { [weak self] p in
            guard let self = self else { return }
            call.resolve(self.estadoDe(p))
        }
    }

    private func avisar() {
        guard let p = reproductor else { return }
        notifyListeners("estado", data: estadoDe(p))
    }

    private func estadoDe(_ p: SonidosPlayer) -> [String: Any] {
        let activo = p.estado == .cargando || p.estado == .listo
        var recetaId: Any = NSNull()
        if activo, let id = p.recetaId {
            recetaId = id
        }
        return [
            "activo": activo,
            "sonando": activo && p.quiereSonar,
            "cargando": p.estado == .cargando,
            "terminado": p.estado == .terminado,
            "recetaId": recetaId,
            "segundos": activo ? p.segundos() : 0.0,
            "duracion": activo ? p.duracion() : 0.0,
            "auriculares": SonidosPlayer.hayAuriculares(),
            "simulado": false
        ]
    }
}
