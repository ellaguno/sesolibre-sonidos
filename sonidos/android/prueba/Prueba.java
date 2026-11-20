import com.sesolibre.sonidos.motor.Nativo;

import java.nio.file.Files;
import java.nio.file.Path;

/** Recorre el puente JNI como lo hace el servicio de Android. Uso: tools/prueba_jni.sh */
public final class Prueba {
    static void exigir(boolean ok, String que) {
        if (!ok) throw new AssertionError(que);
        System.out.println("ok   " + que);
    }

    public static void main(String[] a) throws Exception {
        System.load(Path.of(a[0]).toAbsolutePath().toString());
        Path recetas = Path.of(a[1]);
        Path audio = Path.of(a[2]);
        int CUADROS = 1024;
        float[] bloque = new float[CUADROS * 2];

        exigir(Nativo.validar("{") != null, "una receta rota se rechaza con motivo");
        try {
            Nativo.crear("{}", 0f, -1L, 0L);
            exigir(false, "crear con receta rota debe lanzar");
        } catch (IllegalArgumentException e) {
            exigir(true, "crear con receta rota lanza IllegalArgumentException");
        }

        try (var lista = Files.list(recetas)) {
            for (Path p : lista.filter(x -> x.toString().endsWith(".json")).sorted().toList()) {
                String receta = Files.readString(p);
                exigir(Nativo.validar(receta) == null, p.getFileName() + " valida");
                long banco = Nativo.bancoNuevo();
                int n = 0;
                for (String archivo : Nativo.archivos(receta).split("\n")) {
                    if (archivo.isEmpty()) continue;
                    Nativo.bancoAgregar(banco, archivo, Files.readAllBytes(audio.resolve(archivo)));
                    n++;
                }
                long s = Nativo.crear(receta, 60f, -1L, banco);
                Nativo.bancoLiberar(banco);
                exigir(s != 0 && Math.round(Nativo.duracion(s)) == 60, "  crea la sesión de 60 s con " + n + " muestras");

                double pico = 0;
                int estado = 0;
                for (int i = 0; i < 47 * 20; i++) { // 20 s
                    estado = Nativo.procesar(s, bloque, CUADROS);
                    for (float x : bloque) pico = Math.max(pico, Math.abs(x));
                }
                exigir(estado == 0 && pico > 0.001 && pico <= 0.2001, "  suena bajo techo (pico " + String.format("%.3f", pico) + ")");

                float[] bandas = new float[48];
                Nativo.espectro(s, bandas);
                float mayor = 0;
                for (float b : bandas) mayor = Math.max(mayor, b);
                exigir(mayor > 0.3f && mayor <= 1f, "  el espectro ve el sonido (máx " + String.format("%.2f", mayor) + ")");

                Nativo.atenuar(s, true);
                int bloques = 0;
                while ((Nativo.procesar(s, bloque, CUADROS) & Nativo.EN_SILENCIO) == 0) bloques++;
                Nativo.procesar(s, bloque, CUADROS);
                boolean callado = true;
                for (float x : bloque) callado &= x == 0f;
                exigir(callado && bloques < 47, "  pausa en " + bloques + " bloques y calla del todo");
                Nativo.atenuar(s, false);

                exigir(Nativo.extender(s, 30f) == 30f && Math.round(Nativo.duracion(s)) == 90, "  alargar 30 s alarga la sesión");
                Nativo.volumen(s, -8f);
                Nativo.extender(s, -1f);
                while ((estado & Nativo.TERMINADO) == 0) estado = Nativo.procesar(s, bloque, CUADROS);
                exigir(Math.abs(Nativo.segundos(s) - 90) < 0.1, "  termina sola a los 90 s");
                Nativo.liberar(s);
            }
        }
        exigir(Nativo.procesar(0L, bloque, CUADROS) == -1, "una sesión nula no rompe nada");
        System.out.println("puente JNI: todo bien");
    }
}
