# Integrar Sonidos en otra app

Tres caminos, de más robusto a más simple. Los tres comparten las **recetas** (`recetas/*.json`,
formato en `DISEÑO.md §4`) y sus etiquetas: ninguna app hermana tiene que inventar sonido ni
volver a decidir qué es honesto decir.

## 1. El núcleo dentro de tu servicio (Somnia, de noche)

Para sonar con la pantalla apagada **sin abrir otra app**. Android 12+ no deja arrancar
servicios en primer plano desde el fondo, así que a las 3 a.m. Somnia no puede despertar a
Sonidos: tiene que llevar el núcleo dentro de su propio servicio nocturno.

- Si la app es **Capacitor**: añade el plugin `app/plugin` (`capacitor-sonidos`) como
  dependencia. Trae `libsonidos.so`, `SonidosService` y el puente `Sonidos` (ver
  `app/src/core/motor.ts`). Copia también `assets/audio/` a `public/audio/`.
- Si es **Android nativo**: usa las clases de `app/plugin/android` directamente. Lo mínimo es
  `Nativo` (JNI) más un hilo que escriba en un `AudioTrack`, como hace `SonidosPlayer`:

  ```kotlin
  val banco = Nativo.bancoNuevo()
  for (a in Nativo.archivos(receta).split('\n')) if (a.isNotEmpty())
      Nativo.bancoAgregar(banco, a, assets.open("audio/$a").readBytes())
  val sesion = Nativo.crear(receta, 0f, -1L, banco)   // duración y semilla de la receta
  Nativo.bancoLiberar(banco)
  // en el hilo de audio, hasta que devuelva el bit TERMINADO:
  val estado = Nativo.procesar(sesion, bloque, CUADROS)  // estéreo intercalado, 48 kHz
  ```

- La receta pensada para esto es **`sonar.calma-ronquido`**: diez minutos, techo de −20 dBFS,
  entra en 30 s y sale en 2 min. No hay evidencia de que reduzca ronquidos; Somnia ya los
  cuenta, así que puede comparar noches con y sin ella. Ese es el experimento.

## 2. El núcleo en WebAssembly (Mind, o cualquier web)

Con la pantalla encendida basta la Web Audio API. `npm run wasm` (en `app/`) deja
`public/motor/sonidos.wasm` (660 KB); `public/motor/worklet.js` lo corre en un AudioWorklet
y `src/core/motor-web.ts` es el cliente completo (cargar muestras, tocar, pausar, espectro).
Para llevarlo a otra app: copia esos tres archivos y `assets/audio/` como `audio/`.

La ABI es de C, sin wasm-bindgen (`sonidos/wasm/src/lib.rs`): `reservar`, `validar`,
`archivos`, `banco_agregar`, `crear`, `procesar`, `estado`, `atenuar`, `volumen`, `extender`,
`espectro`, `parar`. El texto sale por `texto_ptr()` / `texto_len()`.

Ideas para Mind: `respirar.seis-por-minuto` un minuto antes de la sesión diaria;
`concentrar.marron-isocronico` durante un puzzle largo, como ajuste opcional.

## 3. Enlaces directos (cualquier app, un atajo, un automatismo)

```
sesolibre-sonidos://tocar?receta=dormir.mar-cuerdas&min=30
```

Abre Sonidos en primer plano y empieza la receta (`min` es opcional). Sirve cuando el
usuario está despierto: la rutina de acostarse, un botón en Somnia, un atajo del sistema.
En la web, la misma consulta (`?receta=…&min=…`) abre la ficha de la receta.

## Compartir diseños

Un diseño del usuario es una receta de la sala `propio`. Viaja como texto en uno de dos
formatos (`app/src/core/diseno.ts`: `codificar`, `decodificar`):

- `sonidos2:` + lo que se eligió en el diseñador, en una lista corta (unas 170 letras). La
  receta se reconstruye al importar y sigue siendo editable.
- `sonidos1:` + la receta entera en base64url (más de mil letras). Se sigue leyendo, y es el
  que se usa para recetas que no nacieron en el diseñador.

Lo que se importa entra siempre como `propio`, con textos neutros, y pasa por la validación
del núcleo: límites de volumen y de frecuencias incluidos.
