# sesolibre-sonidos

Paisajes sonoros con propósito. Eliges una **situación** (dormir, concentrar, meditar,
respirar, calmar) y la app genera en vivo, con la pantalla apagada, el sonido que más
probabilidades tiene de ayudar. Lo que la ciencia respalda va primero; lo alternativo
(solfeggio, 432 Hz, cimática, agua, plantas, sonidos "tipo psicodélico") vive en la misma
app, en la sala **Exploración**, con una etiqueta honesta y la única regla de que no haga daño.

- [DISEÑO.md](DISEÑO.md) — análisis de la evidencia, experiencia, arquitectura y fases.
- [docs/PALABRAS.md](docs/PALABRAS.md) — lo que la app dice y lo que no dice nunca.
- `recetas/` — las recetas (JSON): el contrato entre el núcleo y cualquier interfaz.
- `assets/audio/fase0/` — 90 segundos de cada receta de la Fase 0, para escuchar.
- `sonidos/core` — núcleo en Rust puro: receta + semilla → audio. Sin UI, sin plataforma.
- `sonidos/cli` — `sonidos`: rinde recetas a WAV y las mide.

Hermano de [sesolibre-lienzo](https://github.com/ellaguno/sesolibre-lienzo) (de donde toma
la regla "se escucha antes de seguir" y el manifiesto de licencias) y de
[sesolibre-mind](https://github.com/ellaguno/sesolibre-mind) (de donde tomará el stack de
interfaz y la regla "sin promesas médicas").

## Estado

**Fase 0 — Escuchar: código hecho (19 nov 2026), falta escuchar.** Seis recetas rinden a
WAV, pasan las pruebas (deterministas, bajo techo, sin NaN, sin sobresaltos, sin palabras
prohibidas). Siguiente paso: Eduardo las escucha y decide qué cambia antes de la Fase 1
(servicio Android con pantalla apagada).

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
lentos, afinable), `respiracion` (guía sin voz), `barrido` (solo Exploración) y `ambiente`
(bucles de campo; Fase 2). El núcleo **rechaza** recetas con techo alto, tonos fuera de
20–16 000 Hz, pulsos sin advertencias o textos con palabras prohibidas.

## Licencia

Código: [Apache License 2.0](LICENSE) © 2026 Eduardo Llaguno. Assets: solo CC0, CC-BY,
CC-BY-SA o dominio público, registrados en `assets/manifest.toml`.
