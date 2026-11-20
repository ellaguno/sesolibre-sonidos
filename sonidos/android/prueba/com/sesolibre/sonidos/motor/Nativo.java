package com.sesolibre.sonidos.motor;

/**
 * Gemelo en Java de `Nativo.kt` (app/plugin): mismas firmas, mismo nombre de clase, para
 * probar el puente JNI en una JVM de escritorio sin Android. Si una firma cambia allá y no
 * aquí, `tools/prueba_jni.sh` deja de pasar.
 */
public final class Nativo {
    public static final int TERMINADO = 1;
    public static final int EN_SILENCIO = 2;

    public static native String validar(String receta);
    public static native String archivos(String receta);
    public static native long bancoNuevo();
    public static native void bancoAgregar(long banco, String nombre, byte[] ogg);
    public static native void bancoLiberar(long banco);
    public static native long crear(String receta, float duracionS, long semilla, long banco);
    public static native int procesar(long sesion, float[] salida, int cuadros);
    public static native void atenuar(long sesion, boolean callar);
    public static native void volumen(long sesion, float db);
    public static native float extender(long sesion, float segundos);
    public static native double segundos(long sesion);
    public static native double duracion(long sesion);
    public static native void espectro(long sesion, float[] bandas);
    public static native void liberar(long sesion);

    private Nativo() {}
}
