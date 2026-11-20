/*
 * SesoLibre Sonidos: el núcleo (Rust, sonidos/ios → libsonidos_ios.a) visto desde C y Swift.
 *
 * Tres manejadores opacos:
 *  - SonidosBanco: las muestras Ogg que la receta necesita;
 *  - SonidosSesion: una receta sonando (segura entre hilos: el núcleo la guarda tras un mutex);
 *  - SonidosAnillo: búfer circular estéreo de un productor y un consumidor. El consumidor
 *    (sonidos_anillo_leer) no toma candados ni reserva memoria: es para el hilo de render.
 *
 * Las cadenas entran en UTF-8 terminadas en nulo. Las que salen las reserva el núcleo y se
 * liberan con sonidos_texto_liberar. Los errores salen por `char **error` (puede ser NULL).
 * Audio: 48 000 Hz, estéreo, float.
 */
#ifndef SONIDOS_H
#define SONIDOS_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SonidosBanco SonidosBanco;
typedef struct SonidosSesion SonidosSesion;
typedef struct SonidosAnillo SonidosAnillo;

/* Frecuencia de muestreo de todo el núcleo. */
#define SONIDOS_SR 48000

/* Bits que devuelve sonidos_procesar (0 = suena). */
#define SONIDOS_SUENA 0
#define SONIDOS_TERMINADO 1
#define SONIDOS_EN_SILENCIO 2
#define SONIDOS_ERROR (-1)

/* Libera una cadena que entregó el núcleo. Acepta NULL. */
void sonidos_texto_liberar(char *s);

/* NULL si la receta (JSON) es válida; si no, el motivo. */
char *sonidos_validar(const char *receta);

/* Archivos de public/audio/ que la receta necesita, uno por línea ("" si ninguno).
 * NULL si la receta no vale, con el motivo en *error. */
char *sonidos_archivos(const char *receta, char **error);

SonidosBanco *sonidos_banco_nuevo(void);
/* Decodifica un Ogg y lo agrega con su nombre. 0, o -1 con el motivo en *error. */
int32_t sonidos_banco_agregar(SonidosBanco *banco, const char *nombre, const uint8_t *ogg, size_t largo, char **error);
void sonidos_banco_liberar(SonidosBanco *banco);

/* Crea la sesión. duracion_s <= 0 y semilla < 0 = las de la receta. banco puede ser NULL;
 * si no, sigue siendo del llamador. NULL si falló, con el motivo en *error. */
SonidosSesion *sonidos_crear(const char *receta, float duracion_s, int64_t semilla, const SonidosBanco *banco, char **error);

/* Llena `salida` con `cuadros` cuadros estéreo intercalados (2 * cuadros floats).
 * Devuelve bits SONIDOS_TERMINADO | SONIDOS_EN_SILENCIO, o SONIDOS_ERROR. */
int32_t sonidos_procesar(SonidosSesion *sesion, float *salida, uint32_t cuadros);

/* Pausa (true) o reanuda sin clic: hay que seguir procesando hasta SONIDOS_EN_SILENCIO. */
void sonidos_atenuar(SonidosSesion *sesion, bool callar);
/* Volumen del usuario en dB (0 o menos). */
void sonidos_volumen(SonidosSesion *sesion, float db);
/* "+15 min": devuelve los segundos realmente añadidos. */
float sonidos_extender(SonidosSesion *sesion, float segundos);
double sonidos_segundos(const SonidosSesion *sesion);
double sonidos_duracion(const SonidosSesion *sesion);
/* Tono en vivo del laboratorio: devuelve la frecuencia que quedó (20–2000 Hz). */
float sonidos_mando(SonidosSesion *sesion, float hz);
/* Espectro de lo último que sonó: n valores (hasta 256) de 0 a 1, graves primero. */
void sonidos_espectro(SonidosSesion *sesion, float *bandas, uint32_t n);
void sonidos_liberar(SonidosSesion *sesion);

/* Anillo de `cuadros` cuadros estéreo (la única reserva de memoria). */
SonidosAnillo *sonidos_anillo_nuevo(uint32_t cuadros);
/* Productor: cuadros que caben ahora. */
uint32_t sonidos_anillo_libre(const SonidosAnillo *anillo);
/* Cuadros que esperan a ser leídos. */
uint32_t sonidos_anillo_ocupado(const SonidosAnillo *anillo);
/* Productor: copia hasta `cuadros` cuadros intercalados; devuelve cuántos cupieron. */
uint32_t sonidos_anillo_escribir(const SonidosAnillo *anillo, const float *entrada, uint32_t cuadros);
/* Consumidor (hilo de render): llena izq y der (no intercalados), silencio en lo que falte.
 * Devuelve los cuadros que de verdad había. */
uint32_t sonidos_anillo_leer(const SonidosAnillo *anillo, float *izq, float *der, uint32_t cuadros);
void sonidos_anillo_liberar(SonidosAnillo *anillo);

#ifdef __cplusplus
}
#endif

#endif /* SONIDOS_H */
