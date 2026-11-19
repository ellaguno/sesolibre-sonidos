#!/usr/bin/env bash
# Prueba el puente JNI (sonidos/android) en la JVM de escritorio: compila libsonidos.so
# para esta máquina y recorre todas las recetas como lo hace el servicio de Android.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release -p sonidos-android
OUT="$ROOT/target/prueba-jni"
rm -rf "$OUT" && mkdir -p "$OUT"
# Desde la carpeta de la prueba: javac recorre el directorio actual como classpath.
(cd sonidos/android/prueba && javac -d "$OUT" -cp . com/sesolibre/sonidos/motor/Nativo.java Prueba.java)
java -cp "$OUT" Prueba "$ROOT/target/release/libsonidos.so" "$ROOT/recetas" "$ROOT/assets/audio"
