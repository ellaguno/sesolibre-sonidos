#!/usr/bin/env bash
#
# Compila un APK de depuración (instalable, firmado con la clave debug de Android) en
# android/app/build/outputs/apk/debug/app-debug.apk.
#
#   scripts/android-apk.sh            # compila
#   scripts/android-apk.sh install    # compila e instala por adb
#
# Tres pasos: el núcleo (Rust → libsonidos.so por arquitectura, dentro del plugin), la web
# (Vite → dist/, con las muestras en dist/audio) y el proyecto Android de Capacitor.
# `android/` no se versiona: se genera aquí la primera vez con `npx cap add android`.
#
# Requiere JDK 21, Rust con los targets de Android, cargo-ndk y el SDK de Android con un
# NDK instalado (ANDROID_HOME, por defecto ~/Android/Sdk).

set -euo pipefail
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
# El ícono (assets/icono, generado con tools/iconos.py + iconos.mjs) pisa el de la plantilla.
cp -r native/android/res/. android/app/src/main/res/

# La versión del APK es la de package.json; el código de versión, el número de commits.
VERSION="$(node -p "require('./package.json').version")"
CODIGO="$(git -C "$REPO" rev-list --count HEAD 2>/dev/null || echo 1)"
sed -i -E "s/versionCode [0-9]+/versionCode $CODIGO/; s/versionName \"[^\"]*\"/versionName \"$VERSION\"/" android/app/build.gradle
[ -f android/local.properties ] || echo "sdk.dir=$ANDROID_HOME" > android/local.properties
npx cap sync android

(cd android && ./gradlew assembleDebug --no-daemon -q)
APK="$APP/android/app/build/outputs/apk/debug/app-debug.apk"
echo "APK: $APK ($(du -h "$APK" | cut -f1))"

if [ "${1:-}" = "install" ]; then
  adb install -r "$APK"
  echo "instalado; para ver el registro: adb logcat -s Sonidos AudioTrack MediaSessionService"
fi
