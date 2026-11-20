import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.sesolibre.sonidos',
  appName: 'SesoLibre Sonidos',
  webDir: 'dist',
  // Android se agrega con `npx cap add android` (lo hace scripts/android-apk.sh) e iOS con
  // `npx cap add ios` (lo hace scripts/ios-app.sh, en macOS).
};

export default config;
