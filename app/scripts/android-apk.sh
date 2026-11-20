#!/usr/bin/env bash
#
# Compila la app de Android.
#
#   scripts/android-apk.sh                  # APK de depuración (clave debug de Android)
#   scripts/android-apk.sh install          # … y lo instala por adb
#   scripts/android-apk.sh release [install] # APK de release (R8) firmado con la upload key
#   scripts/android-apk.sh aab              # AAB de release para Google Play
#
# El release se firma con la upload key si están ANDROID_KEYSTORE, ANDROID_KEYSTORE_PASSWORD,
# ANDROID_KEY_ALIAS (y ANDROID_KEY_PASSWORD, si difiere); ver native/android/firma.gradle y
# docs/PUBLICAR.md. Sin ellas sale sin firmar. El AAB firmado queda en
# sesolibre-sonidos-<versión>-<código>.aab (en app/), junto a su mapping.txt de R8.
#
# Tres pasos: el núcleo (Rust → libsonidos.so por arquitectura, dentro del plugin), la web
# (Vite → dist/, con las muestras en dist/audio) y el proyecto Android de Capacitor.
# `android/` no se versiona: se genera aquí la primera vez con `npx cap add android`.
#
# Requiere JDK 21, Rust con los targets de Android, cargo-ndk y el SDK de Android con un
# NDK instalado (ANDROID_HOME, por defecto ~/Android/Sdk).

set -euo pipefail
MODO="${1:-debug}"
APP="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="$(cd "$APP/.." && pwd)"
cd "$APP"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/* | sort -V | tail -1)}"
if [ -z "${JAVA_HOME:-}" ] && [ -d /usr/lib/jvm/java-21-openjdk-amd64 ]; then
  export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
fi

# 1. El núcleo, para teléfonos (arm64, arm de 32 bits) y emulador (x86_64).
command -v cargo-ndk >/dev/null || cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android >/dev/null
(cd "$REPO" && cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 \
  -o "$APP/plugin/android/src/main/jniLibs" build --release -p sonidos-android)

# 2. La web.
[ -d node_modules ] || npm install
npm run build

# 3. Android.
if [ ! -d android ]; then
  npx cap add android
  # Solo vertical.
  sed -i 's/android:launchMode="singleTask"/android:launchMode="singleTask"\n            android:screenOrientation="portrait"/' \
    android/app/src/main/AndroidManifest.xml
  grep -q 'screenOrientation="portrait"' android/app/src/main/AndroidManifest.xml
fi
# Enlaces directos: sesolibre-sonidos://tocar?receta=<id>&min=<minutos>
MANIFIESTO=android/app/src/main/AndroidManifest.xml
if ! grep -q 'android:scheme="sesolibre-sonidos"' "$MANIFIESTO"; then
  sed -i '0,/<\/activity>/s||    <intent-filter>\n                <action android:name="android.intent.action.VIEW" />\n                <category android:name="android.intent.category.DEFAULT" />\n                <category android:name="android.intent.category.BROWSABLE" />\n                <data android:scheme="sesolibre-sonidos" />\n            </intent-filter>\n\n        </activity>|' "$MANIFIESTO"
  grep -q 'android:scheme="sesolibre-sonidos"' "$MANIFIESTO"
fi
# Vertical, pero sin exigir pantalla vertical: si no, Play oculta la app en tablets y PC.
if ! grep -q 'android.hardware.screen.portrait' "$MANIFIESTO"; then
  sed -i 's#^\(\s*\)<application#\1<uses-feature android:name="android.hardware.screen.portrait" android:required="false" />\n\n\1<application#' "$MANIFIESTO"
  grep -q 'android.hardware.screen.portrait" android:required="false"' "$MANIFIESTO"
fi
# El ícono (assets/icono, generado con tools/iconos.py + iconos.mjs) pisa el de la plantilla.
cp -r native/android/res/. android/app/src/main/res/
# Release: R8 (Play Console lo mide en "Optimización") y la firma con la upload key.
GRADLE=android/app/build.gradle
cp native/android/proguard-rules.pro android/app/proguard-rules.pro
cp native/android/firma.gradle android/app/firma.gradle
sed -i "s/minifyEnabled false/minifyEnabled true\n            shrinkResources true/; s/proguard-android\.txt/proguard-android-optimize.txt/" "$GRADLE"
grep -q "apply from: 'firma.gradle'" "$GRADLE" || printf "\napply from: 'firma.gradle'\n" >> "$GRADLE"
grep -q "minifyEnabled true" "$GRADLE"

# La versión del APK es la de package.json; el código de versión, el número de commits.
VERSION="$(node -p "require('./package.json').version")"
CODIGO="$(git -C "$REPO" rev-list --count HEAD 2>/dev/null || echo 1)"
sed -i -E "s/versionCode [0-9]+/versionCode $CODIGO/; s/versionName \"[^\"]*\"/versionName \"$VERSION\"/" android/app/build.gradle
[ -f android/local.properties ] || echo "sdk.dir=$ANDROID_HOME" > android/local.properties
npx cap sync android

firmado() { [ -n "${ANDROID_KEYSTORE:-}" ] || echo " (SIN FIRMAR: faltan ANDROID_KEYSTORE y compañía)"; }
case "$MODO" in
  aab)
    (cd android && ./gradlew bundleRelease --no-daemon -q)
    MAPA=android/app/build/outputs/mapping/release/mapping.txt
    [ -s "$MAPA" ] || { echo "R8 no generó mapping.txt: el AAB no está optimizado" >&2; exit 1; }
    AAB="$APP/android/app/build/outputs/bundle/release/app-release.aab"
    if [ -n "${ANDROID_KEYSTORE:-}" ]; then
      jarsigner -verify "$AAB" | grep -q "jar verified"
      SALIDA="$APP/sesolibre-sonidos-$VERSION-$CODIGO.aab"
      cp "$AAB" "$SALIDA"
      cp "$MAPA" "${SALIDA%.aab}.mapping.txt"
      AAB="$SALIDA"
    fi
    echo "AAB: $AAB ($(du -h "$AAB" | cut -f1)), versionCode $CODIGO$(firmado)"
    ;;
  release)
    (cd android && ./gradlew assembleRelease --no-daemon -q)
    APK="$(ls "$APP"/android/app/build/outputs/apk/release/app-release*.apk | head -1)"
    echo "APK: $APK ($(du -h "$APK" | cut -f1))$(firmado)"
    ;;
  *)
    (cd android && ./gradlew assembleDebug --no-daemon -q)
    APK="$APP/android/app/build/outputs/apk/debug/app-debug.apk"
    echo "APK: $APK ($(du -h "$APK" | cut -f1))"
    ;;
esac

if [ "$MODO" = "install" ] || [ "${2:-}" = "install" ]; then
  adb install -r "$APK"
  echo "instalado; para ver el registro: adb logcat -s Sonidos AudioTrack MediaSessionService"
fi
