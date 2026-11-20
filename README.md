# sesolibre-sonidos

Paisajes sonoros con propósito. Eliges una **situación** (dormir, concentrar, meditar,
respirar, calmar) y la app genera en vivo, con la pantalla apagada, el sonido que más
probabilidades tiene de ayudar. Lo que la ciencia respalda va primero; lo alternativo
(solfeggio, 432 Hz, cimática, agua, plantas, sonidos "tipo psicodélico") vive en la misma
app, en la sala **Exploración**, con una etiqueta honesta y la única regla de que no haga daño.

- [DISEÑO.md](DISEÑO.md) — análisis de la evidencia, experiencia, arquitectura y fases.
- [docs/INTEGRAR.md](docs/INTEGRAR.md) — cómo usar el núcleo desde Somnia, Mind u otra app.
- [docs/PALABRAS.md](docs/PALABRAS.md) — lo que la app dice y lo que no dice nunca.
- `recetas/` — las recetas (JSON): el contrato entre el núcleo y cualquier interfaz.
- `assets/audio/fase0/` — 90 segundos de cada receta de la Fase 0, para escuchar.
- `sonidos/core` — núcleo en Rust puro: receta + semilla → audio. Sin UI, sin plataforma.
- `sonidos/cli` — `sonidos`: rinde recetas a WAV y las mide.
- `sonidos/android` — el núcleo detrás de JNI (`libsonidos.so`).
- `sonidos/wasm` — el núcleo en WebAssembly, para el navegador.
- `app/` — la app (Capacitor + React) y su plugin de Android.
- `assets/icono/` — el ícono (tres variantes) y `tools/iconos.py` + `tools/iconos.mjs` para regenerarlo.
- `assets/audio/` — muestras reales (CC0 o equivalentes) y su índice `banco.json`.

Hermano de [sesolibre-lienzo](https://github.com/ellaguno/sesolibre-lienzo) (de donde toma
la regla "se escucha antes de seguir" y el manifiesto de licencias) y de
[sesolibre-mind](https://github.com/ellaguno/sesolibre-mind) (de donde tomará el stack de
interfaz y la regla "sin promesas médicas").

## Estado

**Fase 0 — Escuchar:** hecha en código; falta el veredicto de oído.

**Fase 1 — Pantalla apagada: construida (20 nov 2026), sin probar en un teléfono.** Ocho
recetas con muestras reales (cuerdas, arpa, vibráfono, agua) y reverberación. App Android
(Capacitor + React) con un plugin propio: el núcleo corre dentro de un `MediaSessionService`
en primer plano, con controles en la notificación y la pantalla de bloqueo, temporizador,
+15 min y modo noche. Compila a APK y pasa lint; el puente JNI pasa `tools/prueba_jni.sh`.
Falta lo que solo dice un teléfono: una noche entera con la pantalla apagada.

Desde el 20 nov: doce recetas (432 Hz, Schumann, lluvia, bosque), un **diseñador** para
armar sonidos propios, guardarlos y compartirlos como un código de texto, y un
**espectrómetro** calculado en Rust. Y un **laboratorio**: cimática con tono en vivo y
cámara, y un experimento de plantas con control y diario.

## App Android

```bash
cd app
npm install
npm run dev                      # la app en el navegador, con sonido (núcleo en WebAssembly)
./scripts/android-apk.sh install # núcleo → web → APK de depuración → adb
```

Requiere JDK 21, Rust, `cargo-ndk` y el SDK de Android con un NDK (`ANDROID_HOME`).
`app/plugin/` es el plugin Capacitor (`capacitor-sonidos`): Kotlin + `libsonidos.so`.

## Uso

```bash
cargo build --release
./target/release/sonidos validar recetas/*.json
./target/release/sonidos render recetas/dormir.rosa-colchon.json /tmp/dormir.wav --segundos 120
./target/release/sonidos lote recetas /tmp/fase0 --segundos 90
cargo test --release
```

`lote` imprime por receta el pico, el RMS y el mayor salto de nivel en 100 ms. Sirven para
encontrar bugs, nunca para decidir si algo ayuda: eso se decide escuchando.

## Recetas

Una receta es JSON versionado (ver `DISEÑO.md §4`). Capas disponibles: `ruido` (blanco,
rosa, marrón, con filtros), `tono`, `pulso` (binaural, isocrónico, AM), `colchon` (acordes
lentos, afinable; sintético o con cuerdas reales), `instrumento` (arpa, vibráfono, flauta…
tocando notas sueltas del acorde), `ambiente` (grabaciones de agua en bucle), `respiracion`
(guía sin voz), `serie` (armónicos de una base), `naturaleza` (viento, lluvia y aves
sintetizados) y `barrido`. Cada capa puede enviar a una reverberación. El núcleo **rechaza** recetas con techo alto, tonos fuera de
20–16 000 Hz, pulsos sin advertencias o textos con palabras prohibidas.

## Licencia

Código: [Apache License 2.0](LICENSE) © 2026 Eduardo Llaguno. Assets: solo CC0, CC-BY,
CC-BY-SA o dominio público, registrados en `assets/manifest.toml`.
