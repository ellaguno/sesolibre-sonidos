package com.sesolibre.sonidos.motor

import android.app.PendingIntent
import android.content.Intent
import android.os.Bundle
import androidx.annotation.OptIn
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.session.DefaultMediaNotificationProvider
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionCommand
import androidx.media3.session.SessionError
import androidx.media3.session.SessionResult
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture

/**
 * El servicio que mantiene el sonido con la pantalla apagada. Media3 lo pasa a primer
 * plano (tipo `mediaPlayback`) mientras suena y lo baja al pausar; la MediaSession da los
 * controles de la notificación, la pantalla de bloqueo y los auriculares.
 *
 * Tiene que arrancar con la app visible (Android 12+ no deja arrancar servicios en primer
 * plano desde el fondo): por eso las recetas las manda el plugin, desde la actividad.
 */
@OptIn(UnstableApi::class)
class SonidosService : MediaSessionService() {
    private var sesion: MediaSession? = null
    private lateinit var player: SonidosPlayer

    override fun onCreate() {
        super.onCreate()
        setMediaNotificationProvider(
            DefaultMediaNotificationProvider.Builder(this).build().apply { setSmallIcon(R.drawable.ic_stat_sonidos) }
        )
        player = SonidosPlayer(this)
        actual = player
        val constructor = MediaSession.Builder(this, player).setCallback(Ordenes())
        packageManager.getLaunchIntentForPackage(packageName)?.let { abrirApp ->
            constructor.setSessionActivity(
                PendingIntent.getActivity(this, 0, abrirApp, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
            )
        }
        sesion = constructor.build()
        player.alError = { mensaje -> sesion?.setSessionExtras(Bundle().apply { putString(Contrato.ERROR, mensaje) }) }
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = sesion

    /** Si quitan la app de recientes mientras suena, sigue sonando: es para dormir. */
    override fun onTaskRemoved(rootIntent: Intent?) {
        val suena = player.playWhenReady &&
            (player.playbackState == Player.STATE_READY || player.playbackState == Player.STATE_BUFFERING)
        if (!suena) stopSelf()
    }

    override fun onDestroy() {
        actual = null
        sesion?.run {
            player.release()
            release()
        }
        sesion = null
        super.onDestroy()
    }

    internal companion object {
        /** El reproductor vivo, para lo que el plugin lee directo (mismo proceso): el espectro. */
        @Volatile
        var actual: SonidosPlayer? = null
    }

    private inner class Ordenes : MediaSession.Callback {
        override fun onConnect(session: MediaSession, controller: MediaSession.ControllerInfo): MediaSession.ConnectionResult {
            val aceptar = MediaSession.ConnectionResult.AcceptedResultBuilder(session)
            // Las órdenes propias (qué receta suena) solo las da nuestra app. El sistema y
            // los auriculares se quedan con play, pausa y parar.
            if (controller.packageName != packageName) return aceptar.build()
            val comandos = MediaSession.ConnectionResult.DEFAULT_SESSION_COMMANDS.buildUpon()
                .add(SessionCommand(Contrato.TOCAR, Bundle.EMPTY))
                .add(SessionCommand(Contrato.EXTENDER, Bundle.EMPTY))
                .add(SessionCommand(Contrato.VOLUMEN, Bundle.EMPTY))
                .build()
            return aceptar.setAvailableSessionCommands(comandos).build()
        }

        override fun onCustomCommand(
            session: MediaSession,
            controller: MediaSession.ControllerInfo,
            customCommand: SessionCommand,
            args: Bundle,
        ): ListenableFuture<SessionResult> {
            if (controller.packageName != packageName) {
                return Futures.immediateFuture(SessionResult(SessionError.ERROR_PERMISSION_DENIED))
            }
            val respuesta = Bundle()
            when (customCommand.customAction) {
                Contrato.TOCAR -> {
                    val receta = args.getString(Contrato.RECETA)
                        ?: return Futures.immediateFuture(SessionResult(SessionError.ERROR_BAD_VALUE))
                    Nativo.validar(receta)?.let { motivo ->
                        respuesta.putString(Contrato.ERROR, motivo)
                        return Futures.immediateFuture(SessionResult(SessionError.ERROR_BAD_VALUE, respuesta))
                    }
                    player.tocar(
                        receta = receta,
                        recetaId = args.getString(Contrato.RECETA_ID, "receta"),
                        titulo = args.getString(Contrato.TITULO, "Sonidos"),
                        subtitulo = args.getString(Contrato.SUBTITULO, ""),
                        duracionS = args.getFloat(Contrato.DURACION_S, 0f),
                        volumenDb = args.getFloat(Contrato.VOLUMEN_DB, 0f),
                    )
                }
                Contrato.EXTENDER -> respuesta.putFloat(Contrato.SEGUNDOS, player.extender(args.getFloat(Contrato.SEGUNDOS, 0f)))
                Contrato.VOLUMEN -> player.volumen(args.getFloat(Contrato.VOLUMEN_DB, 0f))
                else -> return Futures.immediateFuture(SessionResult(SessionError.ERROR_NOT_SUPPORTED))
            }
            return Futures.immediateFuture(SessionResult(SessionResult.RESULT_SUCCESS, respuesta))
        }
    }
}
