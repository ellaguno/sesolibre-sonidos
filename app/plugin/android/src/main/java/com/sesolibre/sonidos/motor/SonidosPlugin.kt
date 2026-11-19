package com.sesolibre.sonidos.motor

import android.content.ComponentName
import android.content.Context
import android.media.AudioDeviceCallback
import android.media.AudioDeviceInfo
import android.media.AudioManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import androidx.core.content.ContextCompat
import androidx.media3.common.C
import androidx.media3.common.Player
import androidx.media3.session.MediaController
import androidx.media3.session.SessionCommand
import androidx.media3.session.SessionResult
import androidx.media3.session.SessionToken
import com.getcapacitor.JSObject
import com.getcapacitor.Plugin
import com.getcapacitor.PluginCall
import com.getcapacitor.PluginMethod
import com.getcapacitor.annotation.CapacitorPlugin
import com.google.common.util.concurrent.ListenableFuture

/**
 * La cara web del servicio: la app (JS) elige la receta y este plugin se la manda al
 * [SonidosService] por un [MediaController]. Todo lo que toca al controlador corre en el
 * hilo principal. Emite el evento `estado` cada vez que algo cambia.
 */
@CapacitorPlugin(name = "Sonidos")
class SonidosPlugin : Plugin() {
    private val principal = Handler(Looper.getMainLooper())
    private var futuro: ListenableFuture<MediaController>? = null
    private var controlador: MediaController? = null
    private var audioManager: AudioManager? = null

    private val oyente = object : Player.Listener {
        override fun onEvents(player: Player, events: Player.Events) = avisar()
    }

    private val alControlador = object : MediaController.Listener {
        override fun onExtrasChanged(controller: MediaController, extras: Bundle) {
            extras.getString(Contrato.ERROR)?.let { notifyListeners("error", JSObject().put("mensaje", it)) }
        }
    }

    private val dispositivos = object : AudioDeviceCallback() {
        override fun onAudioDevicesAdded(added: Array<out AudioDeviceInfo>) = avisar()
        override fun onAudioDevicesRemoved(removed: Array<out AudioDeviceInfo>) = avisar()
    }

    override fun load() {
        audioManager = (context.getSystemService(Context.AUDIO_SERVICE) as AudioManager).also {
            it.registerAudioDeviceCallback(dispositivos, principal)
        }
    }

    override fun handleOnDestroy() {
        audioManager?.unregisterAudioDeviceCallback(dispositivos)
        principal.post {
            controlador?.removeListener(oyente)
            futuro?.let { MediaController.releaseFuture(it) }
            controlador = null
            futuro = null
        }
    }

    /** Conecta con el servicio (lo arranca si hace falta) y entrega el controlador. */
    private fun conControlador(call: PluginCall?, usar: (MediaController) -> Unit) {
        principal.post {
            controlador?.takeIf { it.isConnected }?.let {
                usar(it)
                return@post
            }
            val token = SessionToken(context, ComponentName(context, SonidosService::class.java))
            val f = MediaController.Builder(context, token).setApplicationLooper(Looper.getMainLooper()).setListener(alControlador).buildAsync()
            futuro = f
            f.addListener({
                try {
                    val c = f.get()
                    c.addListener(oyente)
                    controlador = c
                    usar(c)
                } catch (e: Exception) {
                    call?.reject("no se pudo conectar con el servicio de sonido: ${e.message}")
                }
            }, ContextCompat.getMainExecutor(context))
        }
    }

    private fun mandar(call: PluginCall, accion: String, args: Bundle, alResolver: (Bundle) -> JSObject = { JSObject() }) {
        conControlador(call) { c ->
            val r = c.sendCustomCommand(SessionCommand(accion, Bundle.EMPTY), args)
            r.addListener({
                try {
                    val resultado = r.get()
                    if (resultado.resultCode == SessionResult.RESULT_SUCCESS) {
                        call.resolve(alResolver(resultado.extras))
                    } else {
                        call.reject(resultado.extras.getString(Contrato.ERROR) ?: "el servicio rechazó la orden (${resultado.resultCode})")
                    }
                } catch (e: Exception) {
                    call.reject(e.message ?: "falló la orden")
                }
            }, ContextCompat.getMainExecutor(context))
        }
    }

    @PluginMethod
    fun tocar(call: PluginCall) {
        val receta = call.getString("receta") ?: return call.reject("falta la receta")
        Nativo.validar(receta)?.let { return call.reject(it) }
        val minutos = call.getFloat("minutos") ?: 0f
        mandar(call, Contrato.TOCAR, Bundle().apply {
            putString(Contrato.RECETA, receta)
            putString(Contrato.RECETA_ID, call.getString("recetaId") ?: "receta")
            putString(Contrato.TITULO, call.getString("titulo") ?: "Sonidos")
            putString(Contrato.SUBTITULO, call.getString("subtitulo") ?: "")
            putFloat(Contrato.DURACION_S, minutos * 60f)
            putFloat(Contrato.VOLUMEN_DB, call.getFloat("volumenDb") ?: 0f)
        })
    }

    @PluginMethod
    fun pausar(call: PluginCall) = conControlador(call) {
        it.pause()
        call.resolve()
    }

    @PluginMethod
    fun reanudar(call: PluginCall) = conControlador(call) {
        it.play()
        call.resolve()
    }

    @PluginMethod
    fun parar(call: PluginCall) = conControlador(call) {
        it.stop()
        call.resolve()
    }

    @PluginMethod
    fun extender(call: PluginCall) {
        val minutos = call.getFloat("minutos") ?: return call.reject("faltan los minutos")
        mandar(call, Contrato.EXTENDER, Bundle().apply { putFloat(Contrato.SEGUNDOS, minutos * 60f) }) { extras ->
            JSObject().put("minutos", (extras.getFloat(Contrato.SEGUNDOS, 0f) / 60f).toDouble())
        }
    }

    @PluginMethod
    fun volumen(call: PluginCall) {
        val db = call.getFloat("db") ?: return call.reject("faltan los dB")
        mandar(call, Contrato.VOLUMEN, Bundle().apply { putFloat(Contrato.VOLUMEN_DB, db) })
    }

    @PluginMethod
    fun estado(call: PluginCall) = conControlador(call) { call.resolve(estadoDe(it)) }

    private fun avisar() {
        principal.post { notifyListeners("estado", estadoDe(controlador)) }
    }

    private fun estadoDe(c: MediaController?): JSObject {
        val o = JSObject()
        val activo = c != null && c.isConnected &&
            (c.playbackState == Player.STATE_READY || c.playbackState == Player.STATE_BUFFERING)
        o.put("activo", activo)
        o.put("sonando", activo && c!!.playWhenReady)
        o.put("cargando", c != null && c.isConnected && c.playbackState == Player.STATE_BUFFERING)
        o.put("terminado", c != null && c.isConnected && c.playbackState == Player.STATE_ENDED)
        o.put("recetaId", if (activo) c!!.currentMediaItem?.mediaId else null)
        o.put("segundos", if (activo) c!!.currentPosition / 1000.0 else 0.0)
        val duracion = if (activo) c!!.duration else C.TIME_UNSET
        o.put("duracion", if (duracion == C.TIME_UNSET) 0.0 else duracion / 1000.0)
        o.put("auriculares", hayAuriculares())
        o.put("simulado", false)
        return o
    }

    private fun hayAuriculares(): Boolean {
        val tipos = mutableSetOf(
            AudioDeviceInfo.TYPE_WIRED_HEADSET,
            AudioDeviceInfo.TYPE_WIRED_HEADPHONES,
            AudioDeviceInfo.TYPE_BLUETOOTH_A2DP,
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) tipos += AudioDeviceInfo.TYPE_USB_HEADSET
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) tipos += AudioDeviceInfo.TYPE_BLE_HEADSET
        return audioManager?.getDevices(AudioManager.GET_DEVICES_OUTPUTS)?.any { it.type in tipos } ?: false
    }
}
