package com.sesolibre.sonidos.motor

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioTrack
import android.os.Handler
import android.os.Looper
import android.os.PowerManager
import android.os.Process
import android.util.Log
import androidx.annotation.OptIn
import androidx.core.content.ContextCompat
import androidx.media.AudioAttributesCompat
import androidx.media.AudioFocusRequestCompat
import androidx.media.AudioManagerCompat
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.Player
import androidx.media3.common.SimpleBasePlayer
import androidx.media3.common.util.UnstableApi
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture

/**
 * Un [Player] de Media3 cuyo "medio" es una receta que el núcleo genera en vivo. Media3 le
 * pone encima la notificación, la pantalla de bloqueo y los botones de los auriculares.
 *
 * Reglas que cumple:
 * - el audio sale de un hilo propio que escribe en un [AudioTrack] con 200 ms de colchón
 *   (de noche importa no cortar, no la latencia);
 * - pausa y parada bajan el sonido en el núcleo antes de callar: nunca hay clic;
 * - pierde el foco de audio (una llamada, otra app) → pausa; se desconectan los
 *   auriculares → pausa (un binaural no debe seguir por el altavoz a las 3 a.m.);
 * - mientras suena sostiene un wake lock parcial, que suelta al pausar o terminar.
 */
@OptIn(UnstableApi::class)
internal class SonidosPlayer(private val context: Context) : SimpleBasePlayer(Looper.getMainLooper()) {

    /** Se llama (en el hilo principal) si una receta no pudo sonar: falta una muestra, etc. */
    var alError: ((String) -> Unit)? = null

    private val principal = Handler(Looper.getMainLooper())
    private val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
    private val wakeLock = (context.getSystemService(Context.POWER_SERVICE) as PowerManager)
        .newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "sonidos:reproduccion").apply { setReferenceCounted(false) }

    private var hilo: Hilo? = null
    private var item: MediaItem? = null
    private var quiereSonar = false
    private var estado = Player.STATE_IDLE
    private var pausadoPorFoco = false
    private var ruidoRegistrado = false

    private val foco = AudioFocusRequestCompat.Builder(AudioManagerCompat.AUDIOFOCUS_GAIN)
        .setAudioAttributes(
            AudioAttributesCompat.Builder()
                .setUsage(AudioAttributesCompat.USAGE_MEDIA)
                .setContentType(AudioAttributesCompat.CONTENT_TYPE_MUSIC)
                .build()
        )
        .setWillPauseWhenDucked(false)
        .setOnAudioFocusChangeListener { cambio ->
            when (cambio) {
                AudioManager.AUDIOFOCUS_LOSS -> {
                    pausadoPorFoco = false
                    ponerSonando(false)
                }
                AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> if (quiereSonar) {
                    ponerSonando(false)
                    pausadoPorFoco = true
                }
                AudioManager.AUDIOFOCUS_GAIN -> if (pausadoPorFoco) {
                    pausadoPorFoco = false
                    ponerSonando(true)
                }
            }
        }
        .build()

    private val alDesconectar = object : BroadcastReceiver() {
        override fun onReceive(c: Context, intent: Intent) {
            if (intent.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) ponerSonando(false)
        }
    }

    // ------------------------------------------------------------------ órdenes

    /** Empieza una receta (corta la anterior con un desvanecimiento). */
    fun tocar(receta: String, recetaId: String, titulo: String, subtitulo: String, duracionS: Float, volumenDb: Float) {
        hilo?.parar()
        item = MediaItem.Builder()
            .setMediaId(recetaId)
            .setMediaMetadata(
                MediaMetadata.Builder()
                    .setTitle(titulo)
                    .setArtist(subtitulo)
                    .setMediaType(MediaMetadata.MEDIA_TYPE_MUSIC)
                    .setIsPlayable(true)
                    .build()
            )
            .build()
        estado = Player.STATE_BUFFERING
        quiereSonar = pedirFoco()
        pausadoPorFoco = false
        hilo = Hilo(receta, duracionS, volumenDb, pausado = !quiereSonar).also { it.start() }
        ajustarRecursos()
        invalidateState()
    }

    /** "+15 min". Devuelve los segundos realmente añadidos. */
    fun extender(segundos: Float): Float {
        val s = hilo?.extender(segundos) ?: 0f
        if (s > 0f) invalidateState()
        return s
    }

    fun volumen(db: Float) {
        hilo?.volumen(db)
    }

    /** Mueve el tono en vivo del laboratorio. Devuelve la frecuencia que quedó. */
    fun mando(hz: Float): Float = hilo?.mando(hz) ?: 0f

    /** Llena `bandas` con el espectro actual. Devuelve false si no hay nada sonando. */
    fun espectro(bandas: FloatArray): Boolean {
        val h = hilo ?: return false
        if (!quiereSonar || estado != Player.STATE_READY) return false
        return h.espectro(bandas)
    }

    // ------------------------------------------------------------------ SimpleBasePlayer

    override fun getState(): State {
        val comandos = Player.Commands.Builder()
            .addAll(
                Player.COMMAND_PLAY_PAUSE,
                Player.COMMAND_STOP,
                Player.COMMAND_GET_CURRENT_MEDIA_ITEM,
                Player.COMMAND_GET_METADATA,
                Player.COMMAND_GET_TIMELINE,
                Player.COMMAND_GET_AUDIO_ATTRIBUTES,
                Player.COMMAND_RELEASE,
            )
            .build()
        val b = State.Builder()
            .setAvailableCommands(comandos)
            .setAudioAttributes(ATRIBUTOS)
            .setPlayWhenReady(quiereSonar, Player.PLAY_WHEN_READY_CHANGE_REASON_USER_REQUEST)
        val it = item
        val h = hilo
        if (it == null || h == null) {
            return b.setPlaybackState(Player.STATE_IDLE).build()
        }
        val duracionUs = (h.duracionS() * 1_000_000).toLong()
        b.setPlaylist(
            listOf(
                MediaItemData.Builder(it.mediaId)
                    .setMediaItem(it)
                    .setMediaMetadata(it.mediaMetadata)
                    .setDurationUs(if (duracionUs > 0) duracionUs else C.TIME_UNSET)
                    .setIsSeekable(false)
                    .build()
            )
        )
        b.setCurrentMediaItemIndex(0)
        b.setContentPositionMs { (h.segundos() * 1000).toLong() }
        b.setPlaybackState(estado)
        return b.build()
    }

    override fun handleSetPlayWhenReady(playWhenReady: Boolean): ListenableFuture<*> {
        pausadoPorFoco = false
        ponerSonando(playWhenReady)
        return Futures.immediateVoidFuture()
    }

    override fun handlePrepare(): ListenableFuture<*> = Futures.immediateVoidFuture()

    override fun handleStop(): ListenableFuture<*> {
        detener()
        return Futures.immediateVoidFuture()
    }

    override fun handleRelease(): ListenableFuture<*> {
        detener()
        return Futures.immediateVoidFuture()
    }

    // ------------------------------------------------------------------ interno

    private fun ponerSonando(sonar: Boolean) {
        val h = hilo
        if (h == null || estado == Player.STATE_IDLE || estado == Player.STATE_ENDED) {
            quiereSonar = false
        } else if (sonar) {
            quiereSonar = pedirFoco()
            h.pausar(!quiereSonar)
        } else {
            quiereSonar = false
            h.pausar(true)
        }
        ajustarRecursos()
        invalidateState()
    }

    private fun detener() {
        hilo?.parar()
        hilo = null
        item = null
        quiereSonar = false
        pausadoPorFoco = false
        estado = Player.STATE_IDLE
        ajustarRecursos()
        invalidateState()
    }

    private fun pedirFoco(): Boolean =
        AudioManagerCompat.requestAudioFocus(audioManager, foco) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED

    /** Wake lock, foco y aviso de auriculares: solo mientras de verdad suena. */
    private fun ajustarRecursos() {
        val sonando = quiereSonar && (estado == Player.STATE_READY || estado == Player.STATE_BUFFERING)
        if (sonando) {
            val restanteMs = ((hilo?.restanteS() ?: 0.0) * 1000).toLong()
            wakeLock.acquire(restanteMs.coerceAtLeast(60_000L) + 60_000L)
            if (!ruidoRegistrado) {
                ContextCompat.registerReceiver(
                    context, alDesconectar, IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY),
                    ContextCompat.RECEIVER_NOT_EXPORTED,
                )
                ruidoRegistrado = true
            }
        } else {
            if (wakeLock.isHeld) wakeLock.release()
            if (ruidoRegistrado) {
                context.unregisterReceiver(alDesconectar)
                ruidoRegistrado = false
            }
            if (!pausadoPorFoco) AudioManagerCompat.abandonAudioFocusRequest(audioManager, foco)
        }
    }

    private fun alListo(h: Hilo) {
        if (h !== hilo) return
        estado = Player.STATE_READY
        ajustarRecursos()
        invalidateState()
    }

    private fun alTerminar(h: Hilo) {
        if (h !== hilo) return
        estado = Player.STATE_ENDED
        quiereSonar = false
        ajustarRecursos()
        invalidateState()
    }

    private fun alFallar(h: Hilo, mensaje: String) {
        if (h !== hilo) return
        Log.e(TAG, "la receta no pudo sonar: $mensaje")
        hilo = null
        item = null
        quiereSonar = false
        estado = Player.STATE_IDLE
        ajustarRecursos()
        invalidateState()
        alError?.invoke(mensaje)
    }

    /**
     * El hilo de audio de una receta: carga las muestras, crea la sesión en el núcleo y
     * escribe bloques hasta que la receta termina o le piden parar. Es el único que usa el
     * [AudioTrack] y el único que libera la sesión; los demás hilos pasan por [conSesion].
     */
    private inner class Hilo(
        private val receta: String,
        private val duracionS: Float,
        private val volumenDb: Float,
        pausado: Boolean,
    ) : Thread("sonidos-audio") {
        private val candado = Object()
        private var sesion = 0L
        @Volatile private var vivo = true
        private var quierePausa = pausado

        private inline fun <T> conSesion(porDefecto: T, f: (Long) -> T): T =
            synchronized(candado) { if (sesion == 0L) porDefecto else f(sesion) }

        fun segundos(): Double = conSesion(0.0) { Nativo.segundos(it) }
        fun duracionS(): Double = conSesion(0.0) { Nativo.duracion(it) }
        fun restanteS(): Double = conSesion(0.0) { Nativo.duracion(it) - Nativo.segundos(it) }
        fun extender(s: Float): Float = conSesion(0f) { Nativo.extender(it, s) }
        fun volumen(db: Float) = conSesion(Unit) { Nativo.volumen(it, db) }
        fun mando(hz: Float): Float = conSesion(0f) { Nativo.mando(it, hz) }
        fun espectro(bandas: FloatArray): Boolean = conSesion(false) {
            Nativo.espectro(it, bandas)
            true
        }

        fun pausar(pausa: Boolean) = synchronized(candado) {
            quierePausa = pausa
            if (sesion != 0L) Nativo.atenuar(sesion, pausa)
            candado.notifyAll()
        }

        /** Baja el sonido y, en cuanto calla, el hilo sale y libera todo. */
        fun parar() = synchronized(candado) {
            vivo = false
            if (sesion != 0L) Nativo.atenuar(sesion, true)
            candado.notifyAll()
        }

        override fun run() {
            Process.setThreadPriority(Process.THREAD_PRIORITY_AUDIO)
            var track: AudioTrack? = null
            try {
                val creada = crearSesion()
                synchronized(candado) {
                    sesion = creada
                    Nativo.volumen(creada, volumenDb)
                    if (quierePausa || !vivo) Nativo.atenuar(creada, true)
                }
                track = crearTrack()
                principal.post { alListo(this) }
                val bloque = FloatArray(CUADROS * 2)
                track.play()
                while (true) {
                    val e = synchronized(candado) { Nativo.procesar(sesion, bloque, CUADROS) }
                    if (e < 0) throw IllegalStateException("el núcleo falló al procesar")
                    track.write(bloque, 0, bloque.size, AudioTrack.WRITE_BLOCKING)
                    if (e and Nativo.TERMINADO != 0) {
                        principal.post { alTerminar(this) }
                        break
                    }
                    if (e and Nativo.EN_SILENCIO != 0) {
                        if (!vivo) break
                        // Pausa completa: el track se detiene y el hilo duerme hasta reanudar.
                        synchronized(candado) {
                            if (quierePausa && vivo) {
                                track.pause()
                                while (quierePausa && vivo) candado.wait()
                                if (vivo) track.play()
                            }
                        }
                        if (!vivo) break
                    }
                }
            } catch (t: Throwable) {
                val mensaje = t.message ?: t.javaClass.simpleName
                principal.post { alFallar(this, mensaje) }
            } finally {
                try {
                    track?.pause()
                    track?.flush()
                    track?.release()
                } catch (_: Throwable) {
                }
                synchronized(candado) {
                    val s = sesion
                    sesion = 0L
                    if (s != 0L) Nativo.liberar(s)
                }
            }
        }

        private fun crearSesion(): Long {
            val banco = Nativo.bancoNuevo()
            try {
                for (archivo in Nativo.archivos(receta).split('\n')) {
                    if (archivo.isEmpty()) continue
                    if (!vivo) break
                    val bytes = context.assets.open("$RAIZ_AUDIO/$archivo").use { it.readBytes() }
                    Nativo.bancoAgregar(banco, archivo, bytes)
                }
                return Nativo.crear(receta, duracionS, -1L, banco)
            } finally {
                Nativo.bancoLiberar(banco)
            }
        }

        private fun crearTrack(): AudioTrack {
            val formato = AudioFormat.Builder()
                .setSampleRate(SR)
                .setEncoding(AudioFormat.ENCODING_PCM_FLOAT)
                .setChannelMask(AudioFormat.CHANNEL_OUT_STEREO)
                .build()
            val minimo = AudioTrack.getMinBufferSize(SR, AudioFormat.CHANNEL_OUT_STEREO, AudioFormat.ENCODING_PCM_FLOAT)
            val doscientosMs = SR / 5 * 2 * 4
            return AudioTrack.Builder()
                .setAudioAttributes(
                    android.media.AudioAttributes.Builder()
                        .setUsage(android.media.AudioAttributes.USAGE_MEDIA)
                        .setContentType(android.media.AudioAttributes.CONTENT_TYPE_MUSIC)
                        .build()
                )
                .setAudioFormat(formato)
                .setBufferSizeInBytes(maxOf(minimo, doscientosMs))
                .setTransferMode(AudioTrack.MODE_STREAM)
                .build()
        }
    }

    private companion object {
        const val TAG = "Sonidos"
        const val SR = 48_000
        const val CUADROS = 1024
        /** Capacitor copia la web a assets/public; el audio va en public/audio. */
        const val RAIZ_AUDIO = "public/audio"
        val ATRIBUTOS: AudioAttributes = AudioAttributes.Builder()
            .setUsage(C.USAGE_MEDIA)
            .setContentType(C.AUDIO_CONTENT_TYPE_MUSIC)
            .build()
    }
}
