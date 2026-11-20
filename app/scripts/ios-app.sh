#!/usr/bin/env bash
#
# Prepara (y, si se pide, compila) la app de iOS. Solo en macOS, con Xcode.
#
#   scripts/ios-app.sh          # núcleo + web + proyecto ios/ sincronizado (abrir con `npx cap open ios`)
#   scripts/ios-app.sh sim      # … y lo compila para el simulador, sin firmar (para CI)
#
# Tres pasos, como android-apk.sh: el núcleo (Rust → SonidosNucleo.xcframework, dentro del
# plugin), la web (Vite → dist/, con las muestras en dist/audio) y el proyecto iOS de
# Capacitor. `ios/` no se versiona: se genera aquí la primera vez con `npx cap add ios`
# (Capacitor 8 lo arma con Swift Package Manager, sin CocoaPods).

set -euo pipefail
MODO="${1:-}"
APP="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="$(cd "$APP/.." && pwd)"
cd "$APP"

# 1. El núcleo.
"$APP/scripts/ios-nucleo.sh"

# 2. La web.
[ -d node_modules ] || npm install
npm run build

# 3. iOS.
[ -d ios ] || npx cap add ios
PLIST=ios/App/App/Info.plist
PB=/usr/libexec/PlistBuddy
# Sonar con la pantalla apagada: sin esto iOS suspende la app al bloquear.
if ! $PB -c "Print :UIBackgroundModes" "$PLIST" 2>/dev/null | grep -q audio; then
  $PB -c "Add :UIBackgroundModes array" "$PLIST" 2>/dev/null || true
  $PB -c "Add :UIBackgroundModes:0 string audio" "$PLIST"
fi
# Enlaces directos: sesolibre-sonidos://tocar?receta=<id>&min=<minutos>
if ! $PB -c "Print :CFBundleURLTypes" "$PLIST" 2>/dev/null | grep -q sesolibre-sonidos; then
  $PB -c "Add :CFBundleURLTypes array" "$PLIST" 2>/dev/null || true
  $PB -c "Add :CFBundleURLTypes:0 dict" "$PLIST"
  $PB -c "Add :CFBundleURLTypes:0:CFBundleURLName string com.sesolibre.sonidos" "$PLIST"
  $PB -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes array" "$PLIST"
  $PB -c "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string sesolibre-sonidos" "$PLIST"
fi
# Solo vertical en iPhone (en iPad se dejan todas: lo exige la multitarea).
$PB -c "Delete :UISupportedInterfaceOrientations" "$PLIST" 2>/dev/null || true
$PB -c "Add :UISupportedInterfaceOrientations array" "$PLIST"
$PB -c "Add :UISupportedInterfaceOrientations:0 string UIInterfaceOrientationPortrait" "$PLIST"
$PB -c "Print :UIBackgroundModes" "$PLIST" | grep -q audio

# La versión es la de package.json; el número de compilación, el de commits (como Android).
VERSION="$(node -p "require('./package.json').version")"
CODIGO="$(git -C "$REPO" rev-list --count HEAD 2>/dev/null || echo 1)"
PBX=ios/App/App.xcodeproj/project.pbxproj
sed -i '' -E "s/MARKETING_VERSION = [^;]+;/MARKETING_VERSION = $VERSION;/; s/CURRENT_PROJECT_VERSION = [^;]+;/CURRENT_PROJECT_VERSION = $CODIGO;/" "$PBX"

npx cap sync ios

if [ "$MODO" = "sim" ]; then
  xcodebuild -project ios/App/App.xcodeproj -scheme App -configuration Debug \
    -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' \
    -derivedDataPath ios/build ARCHS=arm64 ONLY_ACTIVE_ARCH=NO CODE_SIGNING_ALLOWED=NO build
  echo "compilado para el simulador: ios/build/Build/Products/Debug-iphonesimulator/App.app"
else
  echo "listo: npx cap open ios (firma y archiva desde Xcode)"
fi
