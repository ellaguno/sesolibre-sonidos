import AVFoundation
import Foundation
import MediaPlayer

/// El reproductor de iOS: el equivalente de `SonidosPlayer` + `SonidosService` de Android.
/// Su "medio" es una receta que el núcleo genera en vivo; la pantalla de bloqueo y los
/// auriculares lo controlan por `MPRemoteCommandCenter` y `MPNowPlayingInfoCenter`.
///
/// Reglas que cumple (las mismas que Android):
/// - el audio lo genera un hilo propio que llena un anillo de ~340 ms; el hilo de render de
///   Core Audio solo copia del anillo (de noche importa no cortar, no la latencia);
/// - pausa y parada bajan el sonido en el núcleo antes de callar: nunca hay clic;
/// - una interrupción (una llamada, Siri, otra app) → pausa, y reanuda si iOS lo indica; se
///   desconectan los auriculares → pausa (un binaural no debe seguir por el altavoz a las 3 a.m.);
/// - la sesión de audio (categoría `.playback`, sin mezclar) sigue activa mientras haya una
///   receta cargada, y con `UIBackgroundModes` = `audio` iOS no la suspende con la pantalla
///   apagada.
///
/// Todo lo que no es el hilo de audio corre en el hilo principal.
final class SonidosPlayer {
    enum Estado {
        case reposo, cargando, listo, terminado
    }

    /// Se llama (en el hilo principal) cada vez que algo cambia.
    var alCambiar: (() -> Void)?
    /// Se llama (en el hilo principal) si una receta no pudo sonar: falta una muestra, etc.
    var alError: ((String) -> Void)?

    private(set) var estado: Estado = .reposo
    private(set) var quiereSonar = false
    private(set) var recetaId: String?

    private var motor = AVAudioEngine()
    private var hilo: Hilo?
    /// Hilos con su nodo conectado al motor: el actual y los que aún se desvanecen.
    private var vivos: [Hilo] = []
    private var titulo = "Sonidos"
    private var subtitulo = ""
    private var pausadoPorInterrupcion = false
    private var observadores: [NSObjectProtocol] = []
    private var observadorMotor: NSObjectProtocol?
    private var mandos: [(MPRemoteCommand, Any)] = []

    init() {
        let centro = NotificationCenter.default
        let sesion = AVAudioSession.sharedInstance()
        observadores.append(centro.addObserver(forName: AVAudioSession.interruptionNotification, object: sesion, queue: .main) { [weak self] n in
            self?.alInterrumpir(n)
        })
        observadores.append(centro.addObserver(forName: AVAudioSession.routeChangeNotification, object: sesion, queue: .main) { [weak self] n in
            self?.alCambiarRuta(n)
        })
        observadores.append(centro.addObserver(forName: AVAudioSession.mediaServicesWereResetNotification, object: sesion, queue: .main) { [weak self] _ in
            self?.alReiniciarServicios()
        })
        observarMotor()
        registrarMandos()
    }

    deinit {
        for o in observadores {
            NotificationCenter.default.removeObserver(o)
        }
        if let o = observadorMotor {
            NotificationCenter.default.removeObserver(o)
        }
        for (mando, objetivo) in mandos {
            mando.removeTarget(objetivo)
        }
    }

    // ------------------------------------------------------------------ órdenes

    /// Empieza una receta (corta la anterior con un desvanecimiento).
    func tocar(receta: String, recetaId: String, titulo: String, subtitulo: String, duracionS: Float, volumenDb: Float) {
        hilo?.parar()
        self.recetaId = recetaId
        self.titulo = titulo
        self.subtitulo = subtitulo
        estado = .cargando
        quiereSonar = activarSesion() && arrancarMotor()
        pausadoPorInterrupcion = false
        let h = Hilo(receta: receta, duracionS: duracionS, volumenDb: volumenDb, pausado: !quiereSonar)
        h.alAvisar = { [weak self] hilo, aviso in
            DispatchQueue.main.async { self?.recibir(hilo, aviso) }
        }
        motor.attach(h.nodo)
        motor.connect(h.nodo, to: motor.mainMixerNode, format: Hilo.formato)
        vivos.append(h)
        hilo = h
        h.start()
        ajustarRecursos()
        alCambiar?()
    }

    func pausar() {
        pausadoPorInterrupcion = false
        ponerSonando(false)
    }

    func reanudar() {
        pausadoPorInterrupcion = false
        ponerSonando(true)
    }

    func parar() {
        detener()
    }

    /// "+15 min". Devuelve los segundos realmente añadidos.
    func extender(_ segundos: Float) -> Float {
        let s = hilo?.extender(segundos) ?? 0
        if s > 0 {
            ajustarRecursos()
            alCambiar?()
        }
        return s
    }

    func volumen(_ db: Float) {
        hilo?.volumen(db)
    }

    /// Mueve el tono en vivo del laboratorio. Devuelve la frecuencia que quedó.
    func mando(_ hz: Float) -> Float {
        return hilo?.mando(hz) ?? 0
    }

    /// El espectro actual, o `nil` si no hay nada sonando.
    func espectro(_ n: Int) -> [Float]? {
        guard let h = hilo, quiereSonar, estado == .listo else { return nil }
        return h.espectro(n)
    }

    func segundos() -> Double {
        return hilo?.segundos() ?? 0
    }

    func duracion() -> Double {
        return hilo?.duracionS() ?? 0
    }

    /// Hay auriculares (con cable, Bluetooth o USB) entre las salidas actuales.
    static func hayAuriculares() -> Bool {
        let tipos: Set<AVAudioSession.Port> = [.headphones, .bluetoothA2DP, .bluetoothLE, .bluetoothHFP, .usbAudio]
        return AVAudioSession.sharedInstance().currentRoute.outputs.contains { tipos.contains($0.portType) }
    }

    // ------------------------------------------------------------------ interno

    private func ponerSonando(_ sonar: Bool) {
        if let h = hilo, estado == .cargando || estado == .listo {
            if sonar {
                quiereSonar = activarSesion() && arrancarMotor()
                h.pausar(!quiereSonar)
            } else {
                quiereSonar = false
                h.pausar(true)
            }
        } else {
            quiereSonar = false
        }
        ajustarRecursos()
        alCambiar?()
    }

    private func detener() {
        hilo?.parar()
        hilo = nil
        recetaId = nil
        quiereSonar = false
        pausadoPorInterrupcion = false
        estado = .reposo
        ajustarRecursos()
        alCambiar?()
    }

    /// Categoría `.playback` sin `.mixWithOthers`: así la receta es la que manda en la
    /// pantalla de bloqueo. Devuelve false si iOS no deja sonar ahora (una llamada en curso).
    private func activarSesion() -> Bool {
        let sesion = AVAudioSession.sharedInstance()
        do {
            try sesion.setCategory(.playback, mode: .default, options: [])
            try sesion.setActive(true)
            return true
        } catch {
            NSLog("Sonidos: no se pudo activar la sesión de audio: \(error)")
            return false
        }
    }

    private func desactivarSesion() {
        do {
            try AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
        } catch {
            NSLog("Sonidos: no se pudo soltar la sesión de audio: \(error)")
        }
    }

    private func arrancarMotor() -> Bool {
        if motor.isRunning {
            return true
        }
        motor.prepare()
        do {
            try motor.start()
            return true
        } catch {
            NSLog("Sonidos: no arrancó el motor de audio: \(error)")
            return false
        }
    }

    /// Pantalla de bloqueo, centro de control y sesión: según lo que de verdad pasa.
    private func ajustarRecursos() {
        let centro = MPNowPlayingInfoCenter.default()
        let cargada = estado == .cargando || estado == .listo
        if cargada || estado == .terminado {
            var info: [String: Any] = [
                MPMediaItemPropertyTitle: titulo,
                MPMediaItemPropertyArtist: subtitulo,
                MPNowPlayingInfoPropertyMediaType: MPNowPlayingInfoMediaType.audio.rawValue,
                MPNowPlayingInfoPropertyIsLiveStream: false,
                MPNowPlayingInfoPropertyElapsedPlaybackTime: segundos(),
                MPNowPlayingInfoPropertyPlaybackRate: (cargada && quiereSonar) ? 1.0 : 0.0,
            ]
            let d = duracion()
            if d > 0 {
                info[MPMediaItemPropertyPlaybackDuration] = d
            }
            centro.nowPlayingInfo = info
        } else {
            centro.nowPlayingInfo = nil
        }
        let remoto = MPRemoteCommandCenter.shared()
        remoto.playCommand.isEnabled = cargada
        remoto.pauseCommand.isEnabled = cargada
        remoto.togglePlayPauseCommand.isEnabled = cargada
        remoto.stopCommand.isEnabled = cargada
    }

    private func recibir(_ h: Hilo, _ aviso: Hilo.Aviso) {
        switch aviso {
        case .listo:
            guard h === hilo else { return }
            estado = .listo
            ajustarRecursos()
            alCambiar?()
        case .callado:
            // Pausa completa: el motor se detiene (no gasta batería) hasta reanudar.
            guard h === hilo, !quiereSonar, vivos.allSatisfy({ $0 === h }) else { return }
            motor.pause()
        case .terminado:
            guard h === hilo else { return }
            estado = .terminado
            quiereSonar = false
            ajustarRecursos()
            alCambiar?()
        case .fallo(let mensaje):
            guard h === hilo else { return }
            NSLog("Sonidos: la receta no pudo sonar: \(mensaje)")
            hilo = nil
            recetaId = nil
            quiereSonar = false
            estado = .reposo
            ajustarRecursos()
            alCambiar?()
            alError?(mensaje)
        case .salio:
            soltar(h)
        }
    }

    /// El hilo terminó: se desconecta su nodo y, con un margen para el último ciclo de render,
    /// se libera su anillo. Si ya nada suena, se suelta el motor y la sesión.
    private func soltar(_ h: Hilo) {
        vivos.removeAll { $0 === h }
        // Tras un reinicio de los servicios el nodo era de un motor que ya no existe.
        if h.nodo.engine === motor {
            motor.disconnectNodeOutput(h.nodo)
            motor.detach(h.nodo)
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            h.liberarAnillo()
        }
        if vivos.isEmpty && (hilo == nil || estado == .terminado) {
            motor.stop()
            if !pausadoPorInterrupcion {
                desactivarSesion()
            }
        }
    }

    private func alInterrumpir(_ n: Notification) {
        guard let info = n.userInfo,
              let valor = info[AVAudioSessionInterruptionTypeKey] as? UInt,
              let tipo = AVAudioSession.InterruptionType(rawValue: valor) else { return }
        switch tipo {
        case .began:
            if quiereSonar {
                ponerSonando(false)
                pausadoPorInterrupcion = true
            }
        case .ended:
            let opciones = AVAudioSession.InterruptionOptions(rawValue: (info[AVAudioSessionInterruptionOptionKey] as? UInt) ?? 0)
            if pausadoPorInterrupcion && opciones.contains(.shouldResume) {
                pausadoPorInterrupcion = false
                ponerSonando(true)
            }
            pausadoPorInterrupcion = false
        @unknown default:
            break
        }
    }

    private func alCambiarRuta(_ n: Notification) {
        if let valor = n.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt,
           AVAudioSession.RouteChangeReason(rawValue: valor) == .oldDeviceUnavailable {
            // Se desconectaron los auriculares: pausa (como ACTION_AUDIO_BECOMING_NOISY).
            pausadoPorInterrupcion = false
            ponerSonando(false)
        } else {
            alCambiar?()
        }
    }

    /// El motor se detiene solo si cambia la configuración (otra frecuencia de salida, otra
    /// ruta). Si la receta debía sonar, se vuelve a arrancar.
    private func observarMotor() {
        if let o = observadorMotor {
            NotificationCenter.default.removeObserver(o)
        }
        observadorMotor = NotificationCenter.default.addObserver(forName: .AVAudioEngineConfigurationChange, object: motor, queue: .main) { [weak self] _ in
            guard let self = self else { return }
            if self.quiereSonar && !self.motor.isRunning && !self.arrancarMotor() {
                self.ponerSonando(false)
            }
        }
    }

    /// Los servicios de audio de iOS se reiniciaron: el motor ya no sirve. Se para todo y se
    /// arma uno nuevo; la receta hay que volver a tocarla.
    private func alReiniciarServicios() {
        let habia = hilo != nil
        // Los hilos callan solos (nadie lee su anillo) y avisan `salio` como siempre.
        for h in vivos {
            h.parar()
        }
        hilo = nil
        recetaId = nil
        quiereSonar = false
        pausadoPorInterrupcion = false
        estado = .reposo
        motor = AVAudioEngine()
        observarMotor()
        ajustarRecursos()
        alCambiar?()
        if habia {
            alError?("se reiniciaron los servicios de audio de iOS")
        }
    }

    /// Play, pausa y parar desde la pantalla de bloqueo, el centro de control y los auriculares.
    private func registrarMandos() {
        let c = MPRemoteCommandCenter.shared()
        mandos.append((c.playCommand, c.playCommand.addTarget { [weak self] _ in
            guard let self = self, self.hilo != nil else { return .noActionableNowPlayingItem }
            self.reanudar()
            return .success
        }))
        mandos.append((c.pauseCommand, c.pauseCommand.addTarget { [weak self] _ in
            guard let self = self, self.hilo != nil else { return .noActionableNowPlayingItem }
            self.pausar()
            return .success
        }))
        mandos.append((c.togglePlayPauseCommand, c.togglePlayPauseCommand.addTarget { [weak self] _ in
            guard let self = self, self.hilo != nil else { return .noActionableNowPlayingItem }
            if self.quiereSonar {
                self.pausar()
            } else {
                self.reanudar()
            }
            return .success
        }))
        mandos.append((c.stopCommand, c.stopCommand.addTarget { [weak self] _ in
            guard let self = self else { return .commandFailed }
            self.parar()
            return .success
        }))
        // Sin adelantar, retroceder ni saltar: una receta no tiene pistas ni se puede buscar.
        c.changePlaybackPositionCommand.isEnabled = false
        c.nextTrackCommand.isEnabled = false
        c.previousTrackCommand.isEnabled = false
        c.skipForwardCommand.isEnabled = false
        c.skipBackwardCommand.isEnabled = false
        ajustarRecursos()
    }
}

/// El hilo de audio de una receta: carga las muestras, crea la sesión en el núcleo y llena el
/// anillo hasta que la receta termina o le piden parar. Es el único que escribe en el anillo y
/// el único que libera la sesión; los demás hilos pasan por `conSesion`. El nodo de render
/// solo lee del anillo.
final class Hilo: Thread {
    enum Aviso {
        case listo, callado, terminado, salio
        case fallo(String)
    }

    static let sr: Double = 48_000
    static let cuadros: UInt32 = 1024
    /// ~340 ms de colchón: con la pantalla apagada iOS pide bloques de hasta 4096 cuadros.
    static let capacidad: UInt32 = 16_384
    /// Cada cuánto mira el productor si hay sitio en el anillo.
    static let siesta: TimeInterval = 0.02
    /// Si en este tiempo nadie lee del anillo, el motor de audio está parado.
    static let paciencia: TimeInterval = 0.5
    /// Estéreo, float, no intercalado (el formato estándar de AVAudioEngine) a 48 kHz.
    static let formato: AVAudioFormat = AVAudioFormat(standardFormatWithSampleRate: 48_000, channels: 2)!

    /// Se llama desde el hilo de audio; quien lo pone se encarga de pasar al principal.
    var alAvisar: ((Hilo, Aviso) -> Void)?

    let nodo: AVAudioSourceNode
    private let anillo: OpaquePointer
    private let receta: String
    private let duracionS: Float
    private let volumenDb: Float
    private let candado = NSCondition()
    private var sesion: OpaquePointer?
    private var vivo = true
    private var quierePausa: Bool
    /// Nadie consume del anillo (motor detenido): se descarta y se sigue al ritmo del reloj.
    private var atascado = false
    private var anilloSuelto = false

    init(receta: String, duracionS: Float, volumenDb: Float, pausado: Bool) {
        self.receta = receta
        self.duracionS = duracionS
        self.volumenDb = volumenDb
        self.quierePausa = pausado
        let a = Nativo.anilloNuevo(Hilo.capacidad)!
        self.anillo = a
        // El bloque de render solo copia del anillo: ni candados, ni memoria, ni Swift
        // con conteo de referencias (captura un puntero, nada más).
        self.nodo = AVAudioSourceNode(format: Hilo.formato) { silencio, _, cuadros, lista -> OSStatus in
            let buffers = UnsafeMutableAudioBufferListPointer(lista)
            guard buffers.count >= 2,
                  let izq = buffers[0].mData?.assumingMemoryBound(to: Float.self),
                  let der = buffers[1].mData?.assumingMemoryBound(to: Float.self) else {
                return noErr
            }
            if Nativo.anilloLeer(a, izq, der, cuadros) == 0 {
                silencio.pointee = true
            }
            return noErr
        }
        super.init()
        name = "sonidos-audio"
        qualityOfService = .userInteractive
    }

    private func conSesion<T>(_ porDefecto: T, _ f: (OpaquePointer) -> T) -> T {
        candado.lock()
        defer { candado.unlock() }
        guard let s = sesion else { return porDefecto }
        return f(s)
    }

    func segundos() -> Double { return conSesion(0) { Nativo.segundos($0) } }
    func duracionS() -> Double { return conSesion(0) { Nativo.duracion($0) } }
    func extender(_ s: Float) -> Float { return conSesion(0) { Nativo.extender($0, s) } }
    func volumen(_ db: Float) { conSesion(()) { Nativo.volumen($0, db) } }
    func mando(_ hz: Float) -> Float { return conSesion(0) { Nativo.mando($0, hz) } }
    func espectro(_ n: Int) -> [Float]? { return conSesion(nil as [Float]?) { s -> [Float]? in Nativo.espectro(s, n) } }

    func pausar(_ pausa: Bool) {
        candado.lock()
        quierePausa = pausa
        if let s = sesion {
            Nativo.atenuar(s, pausa)
        }
        candado.broadcast()
        candado.unlock()
    }

    /// Baja el sonido y, en cuanto calla, el hilo sale y libera todo.
    func parar() {
        candado.lock()
        vivo = false
        if let s = sesion {
            Nativo.atenuar(s, true)
        }
        candado.broadcast()
        candado.unlock()
    }

    /// Lo llama el principal después de `salio` (el hilo ya no escribe) y de quitar el nodo
    /// del motor (el render ya no lee), con un margen por el último ciclo de render.
    func liberarAnillo() {
        candado.lock()
        defer { candado.unlock() }
        if anilloSuelto { return }
        anilloSuelto = true
        Nativo.anilloLiberar(anillo)
    }

    private func estaVivo() -> Bool {
        candado.lock()
        defer { candado.unlock() }
        return vivo
    }

    private func avisar(_ a: Aviso) {
        alAvisar?(self, a)
    }

    override func main() {
        var fallo: String? = nil
        var bloque = [Float](repeating: 0, count: Int(Hilo.cuadros) * 2)
        do {
            let creada = try crearSesion()
            candado.lock()
            sesion = creada
            Nativo.volumen(creada, volumenDb)
            if quierePausa || !vivo {
                Nativo.atenuar(creada, true)
            }
            candado.unlock()
            avisar(.listo)
            while true {
                esperarSitio()
                candado.lock()
                let e: Int32 = bloque.withUnsafeMutableBufferPointer { b in
                    Nativo.procesar(creada, b.baseAddress!, Hilo.cuadros)
                }
                candado.unlock()
                if e < 0 {
                    throw Nativo.Fallo(mensaje: "el núcleo falló al procesar")
                }
                bloque.withUnsafeBufferPointer { b in
                    _ = Nativo.anilloEscribir(anillo, b.baseAddress!, Hilo.cuadros)
                }
                if e & Nativo.terminado != 0 {
                    drenar()
                    avisar(.terminado)
                    break
                }
                if e & Nativo.enSilencio != 0 {
                    if !estaVivo() {
                        drenar()
                        break
                    }
                    // Pausa completa: se deja sonar lo que queda en el anillo (el final del
                    // desvanecimiento), se avisa para detener el motor y el hilo duerme.
                    candado.lock()
                    let pausa = quierePausa
                    candado.unlock()
                    if pausa {
                        drenar()
                        avisar(.callado)
                        candado.lock()
                        while quierePausa && vivo {
                            candado.wait()
                        }
                        candado.unlock()
                        atascado = false
                    }
                    if !estaVivo() {
                        break
                    }
                }
            }
        } catch let f as Nativo.Fallo {
            fallo = f.mensaje
        } catch {
            fallo = error.localizedDescription
        }
        candado.lock()
        if let s = sesion {
            sesion = nil
            Nativo.liberar(s)
        }
        candado.unlock()
        if let f = fallo {
            avisar(.fallo(f))
        }
        avisar(.salio)
    }

    private func crearSesion() throws -> OpaquePointer {
        guard let banco = Nativo.bancoNuevo() else {
            throw Nativo.Fallo(mensaje: "no hay memoria para el banco")
        }
        defer { Nativo.bancoLiberar(banco) }
        // Capacitor copia la web (dist/) a public/ dentro del paquete de la app.
        let raiz = Bundle.main.bundleURL.appendingPathComponent(Hilo.raizAudio, isDirectory: true)
        for archivo in try Nativo.archivos(receta) {
            if !estaVivo() { break }
            let url = raiz.appendingPathComponent(archivo)
            let datos: Data
            do {
                datos = try Data(contentsOf: url, options: .mappedIfSafe)
            } catch {
                throw Nativo.Fallo(mensaje: "falta la muestra \(Hilo.raizAudio)/\(archivo)")
            }
            try Nativo.bancoAgregar(banco, archivo, datos)
        }
        return try Nativo.crear(receta, duracionS, -1, banco)
    }

    /// Espera a que quepa un bloque. Si nadie lee del anillo (el motor está detenido por una
    /// interrupción o un cambio de ruta), no espera: tira el audio y sigue al ritmo del reloj,
    /// para que la pausa o la parada lleguen a término.
    private func esperarSitio() {
        if atascado {
            if Nativo.anilloLibre(anillo) >= Hilo.cuadros {
                atascado = false
            } else {
                Thread.sleep(forTimeInterval: Double(Hilo.cuadros) / Hilo.sr)
                return
            }
        }
        var quieto: TimeInterval = 0
        var antes = Nativo.anilloOcupado(anillo)
        while Nativo.anilloLibre(anillo) < Hilo.cuadros {
            Thread.sleep(forTimeInterval: Hilo.siesta)
            let ahora = Nativo.anilloOcupado(anillo)
            if ahora < antes {
                quieto = 0
                antes = ahora
            } else {
                quieto += Hilo.siesta
                if quieto >= Hilo.paciencia {
                    atascado = true
                    return
                }
            }
        }
    }

    /// Espera a que suene lo que queda en el anillo (o a que se vea que nadie lo leerá).
    private func drenar() {
        var quieto: TimeInterval = 0
        var antes = Nativo.anilloOcupado(anillo)
        while antes > 0 && quieto < Hilo.paciencia {
            Thread.sleep(forTimeInterval: Hilo.siesta)
            let ahora = Nativo.anilloOcupado(anillo)
            if ahora < antes {
                quieto = 0
            } else {
                quieto += Hilo.siesta
            }
            antes = ahora
        }
    }

    private static let raizAudio = "public/audio"
}
