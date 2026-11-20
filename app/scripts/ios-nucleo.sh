#!/usr/bin/env bash
#
# Compila el núcleo (Rust, sonidos/ios) para iOS y lo empaqueta como XCFramework estático en
# plugin/ios/Frameworks/SonidosNucleo.xcframework, donde lo buscan plugin/Package.swift
# (binaryTarget) y plugin/CapacitorSonidos.podspec (vendored_frameworks).
#
#   scripts/ios-nucleo.sh            # release (lo normal)
#   scripts/ios-nucleo.sh debug      # sin optimizar (solo para depurar el núcleo)
#
# Dos rebanadas: teléfonos (aarch64-apple-ios) y simulador en Mac con Apple Silicon
# (aarch64-apple-ios-sim). Con SONIDOS_SIM_X86=1 el simulador lleva también x86_64 (Mac Intel).
#
# Solo corre en macOS (xcodebuild). Requiere Rust (rustup) y las Xcode Command Line Tools.
# El XCFramework no se versiona (como los .so de Android): se genera aquí.

set -euo pipefail
MODO="${1:-release}"
APP="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="$(cd "$APP/.." && pwd)"
SALIDA="$APP/plugin/ios/Frameworks"
MARCO="$SALIDA/SonidosNucleo.xcframework"
LIB=libsonidos_ios.a

if [ "$(uname -s)" != "Darwin" ]; then
  echo "ios-nucleo.sh necesita macOS (xcodebuild -create-xcframework)" >&2
  exit 1
fi

# El mismo mínimo que Capacitor 8 (iOS 15): si no, el enlazador avisa de objetos "más nuevos".
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-15.0}"

DESTINOS=(aarch64-apple-ios aarch64-apple-ios-sim)
[ "${SONIDOS_SIM_X86:-0}" = "1" ] && DESTINOS+=(x86_64-apple-ios)
rustup target add "${DESTINOS[@]}" >/dev/null

PERFIL=release
BANDERA=(--release)
if [ "$MODO" = "debug" ]; then
  PERFIL=debug
  BANDERA=()
fi

for t in "${DESTINOS[@]}"; do
  # (la forma ${a[@]+…} aguanta el arreglo vacío con set -u en el bash 3.2 de macOS)
  (cd "$REPO" && cargo build ${BANDERA[@]+"${BANDERA[@]}"} -p sonidos-ios --target "$t")
done

OBJ="$REPO/target"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Las cabeceras van en una subcarpeta con el nombre del módulo: así el module.modulemap no
# choca con el de otras bibliotecas cuando Xcode junta todas en include/.
mkdir -p "$TMP/Headers/SonidosNucleo"
cp "$REPO/sonidos/ios/include/sonidos.h" "$REPO/sonidos/ios/include/module.modulemap" "$TMP/Headers/SonidosNucleo/"

# Simulador: una sola biblioteca con todas sus arquitecturas.
SIM="$OBJ/aarch64-apple-ios-sim/$PERFIL/$LIB"
if [ "${SONIDOS_SIM_X86:-0}" = "1" ]; then
  mkdir -p "$TMP/sim"
  lipo -create "$SIM" "$OBJ/x86_64-apple-ios/$PERFIL/$LIB" -output "$TMP/sim/$LIB"
  SIM="$TMP/sim/$LIB"
fi

rm -rf "$MARCO"
mkdir -p "$SALIDA"
xcodebuild -create-xcframework \
  -library "$OBJ/aarch64-apple-ios/$PERFIL/$LIB" -headers "$TMP/Headers" \
  -library "$SIM" -headers "$TMP/Headers" \
  -output "$MARCO"

echo "XCFramework: $MARCO ($(du -sh "$MARCO" | cut -f1))"
