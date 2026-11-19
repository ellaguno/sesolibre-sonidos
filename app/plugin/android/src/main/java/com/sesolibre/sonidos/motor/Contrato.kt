package com.sesolibre.sonidos.motor

/** Órdenes propias que el plugin manda al servicio por la MediaSession. */
internal object Contrato {
    const val TOCAR = "com.sesolibre.sonidos.TOCAR"
    const val EXTENDER = "com.sesolibre.sonidos.EXTENDER"
    const val VOLUMEN = "com.sesolibre.sonidos.VOLUMEN"

    const val RECETA = "receta"
    const val RECETA_ID = "recetaId"
    const val TITULO = "titulo"
    const val SUBTITULO = "subtitulo"
    const val DURACION_S = "duracionS"
    const val VOLUMEN_DB = "volumenDb"
    const val SEGUNDOS = "segundos"
    const val ERROR = "error"
}
