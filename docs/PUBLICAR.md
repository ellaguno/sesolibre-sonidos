# Publicar en Google Play y App Store

La app sale en las dos tiendas con las mismas llaves que el resto de SesoLibre: la
**upload key** de Android de Games y Mind (`~/.sesolibre-keys/upload-keystore.jks`, alias
`upload`) y el certificado **Apple Distribution** del equipo `PLNS58WDQF` (vence en nov 2027).
Las fichas se escriben según [PALABRAS.md](PALABRAS.md): nada de curar, tratar ni terapia.

| | Android | iOS |
|---|---|---|
| Id | `com.sesolibre.sonidos` | `com.sesolibre.sonidos` |
| Nombre | SesoLibre Sonidos | SesoLibre Sonidos |
| Categoría | Salud y deportes | Health & Fitness (2.ª: Lifestyle) |
| Política de privacidad | <https://ellaguno.github.io/sesolibre-sonidos/privacidad.html> | la misma |
| Workflow | `.github/workflows/android.yml` | `.github/workflows/ios.yml` |

## Android

### En local

```bash
cd app
C=~/.sesolibre-keys/credentials.txt
export ANDROID_KEYSTORE=~/.sesolibre-keys/upload-keystore.jks \
  ANDROID_KEY_ALIAS="$(sed -n 's/^Alias: *//p' $C)" \
  ANDROID_KEYSTORE_PASSWORD="$(sed -n 's/^STORE_PASS: *//p' $C)" \
  ANDROID_KEY_PASSWORD="$(sed -n 's/^KEY_PASS: *//p' $C)"
./scripts/android-apk.sh release install   # el build de tienda, en el teléfono
./scripts/android-apk.sh aab               # → sesolibre-sonidos-<versión>-<código>.aab
```

`versionCode` = número de commits (siempre crece); `versionName` = `version` de `package.json`.
El release pasa por R8; si algo de JNI o Capacitor deja de responder solo en release, la regla
falta en `app/native/android/proguard-rules.pro`.

### En CI

Cada tag `v*` (o *Actions → Android (Play) → Run workflow*) compila el AAB firmado y lo sube
como **borrador** a la pista `alpha` (prueba cerrada). Secrets del repo:

```bash
R=ellaguno/sesolibre-sonidos; C=~/.sesolibre-keys/credentials.txt
base64 -w0 ~/.sesolibre-keys/upload-keystore.jks | gh secret set ANDROID_KEYSTORE_BASE64 -R $R
sed -n 's/^STORE_PASS: *//p' $C | tr -d '\n' | gh secret set ANDROID_KEYSTORE_PASSWORD -R $R
sed -n 's/^Alias: *//p'      $C | tr -d '\n' | gh secret set ANDROID_KEY_ALIAS -R $R
sed -n 's/^KEY_PASS: *//p'   $C | tr -d '\n' | gh secret set ANDROID_KEY_PASSWORD -R $R
gh secret set PLAY_SERVICE_ACCOUNT_JSON -R $R < ~/apple-certs/sesolibre-play-a055c2d4d047.json
```

La cuenta de servicio necesita acceso a esta app en *Play Console → Usuarios y permisos*.
Google no deja subir por API a una app nueva: **el primer AAB se sube a mano** en la consola.

### Primera vez en Play Console

1. *Crear app*: nombre, idioma predeterminado **español (Latinoamérica)**, app, gratuita.
2. Inscribir **Play App Signing** (Google firma; lo nuestro es solo la llave de subida).
3. **Política de privacidad**: la URL de arriba.
4. **Acceso a la app**: todo disponible sin restricciones.
5. **Anuncios**: no contiene.
6. **Clasificación de contenido** (IARC): categoría *Referencia, salud y bienestar*; todo "no".
7. **Público objetivo**: 18+ (no dirigida a niños).
8. **Seguridad de los datos**: *No se recopilan datos* y *no se comparten*. La cámara se usa
   en el dispositivo y no sale de él, así que no cuenta como recopilación.
9. **Servicio en primer plano** (`FOREGROUND_SERVICE_MEDIA_PLAYBACK`): tipo *Reproducción de
   contenido multimedia*. Uso: *"El usuario elige un sonido y lo inicia; el servicio lo genera
   y reproduce con la pantalla apagada hasta que termina el temporizador o el usuario lo detiene
   desde la notificación."* Hace falta un video corto (iniciar → bloquear → notificación).
10. **App de salud**: declarar *Bienestar / relajación y sueño*, sin funciones médicas.
11. Ficha: `node app/scripts/play.mjs ficha docs/ficha-play.json assets/play` sube textos,
    ícono, gráfico de funciones y capturas (los gráficos los genera `tools/play-graficos.py`;
    las capturas salen del teléfono con adb, recortadas a ≤ 2:1). Categoría: *Salud y deportes*.
12. *Prueba cerrada* (pista `alpha`): subir el AAB, crear la lista de testers y compartir el
    enlace. Cuenta personal: **12 testers con opt-in durante 14 días** antes de pedir producción
    (ver `sesolibre-games/docs/RELEASING.md`).

## iOS

Lo compila y sube `.github/workflows/ios.yml` en un runner de macOS: núcleo Rust →
XCFramework, `cap add ios`, plugin nativo (AVAudioEngine + pantalla de bloqueo), firma y
TestFlight (*Actions → iOS (TestFlight) → Run workflow*, o cada tag `v*`). La app ya existe en
App Store Connect (id `6837455986`); el Bundle ID y el perfil *SesoLibre Sonidos App Store* los
crea `app/scripts/asc-profile.py` por la API. Secrets (los mismos valores que Games):

```bash
R=ellaguno/sesolibre-sonidos; A=~/apple-certs
printf PLNS58WDQF | gh secret set APPLE_TEAM_ID -R $R
base64 -w0 $A/dist-ci.p12 | gh secret set APPLE_CERT_P12_BASE64 -R $R
tr -d '\n' < $A/dist-ci.p12.password | gh secret set APPLE_CERT_PASSWORD -R $R
openssl rand -hex 16 | tr -d '\n' | gh secret set KEYCHAIN_PASSWORD -R $R
printf 596256L8PU | gh secret set ASC_KEY_ID -R $R
printf 12352a5c-64b1-4d69-8a4f-374ce2bb62f8 | gh secret set ASC_ISSUER_ID -R $R
gh secret set ASC_PRIVATE_KEY -R $R < $A/AuthKey_596256L8PU.p8
```

**TestFlight abierto.** Tras subir, el flujo corre `app/scripts/asc-testflight.py`: espera a que
Apple procese la compilación, la pone en el grupo externo *Abierto* (variable `TESTFLIGHT_GRUPO`),
la manda a revisión beta y activa el **enlace público**, que queda en el resumen del run. El
enlace funciona cuando Apple aprueba esa revisión (suele ser un día). Requisito manual, una
vez: *TestFlight → Información de la prueba* con descripción, correo y contacto.

En local (macOS): `app/scripts/ios-app.sh` prepara `app/ios/` y `npx cap open ios`.

Ficha del App Store: `python3 app/scripts/asc-ficha.py docs/ficha-appstore.json assets/appstore`
sube categorías, clasificación por edad, textos (es-MX, en-US), capturas y asigna la última
compilación procesada. Las capturas (iPhone 6,9" e iPad 13") salen de la web con
`node tools/capturas-ios.mjs` (con `npx vite preview --port 4173` corriendo en `app/`).
En la web quedaron hechos a mano: *Privacidad de la app* (no se recopilan datos), disponibilidad
en 175 países y los datos de contacto para la revisión (los de Mind). Precio: 0.99 USD (base EE. UU.,
como Games y Mind); en Google Play es gratis, y Play no deja pasar una app gratuita a de pago.

## Ficha

**Descripción corta (≤ 80)**

- es: `Sonidos para dormir, concentrarte o calmarte, en vivo y con la pantalla apagada.`
- en: `Sounds to sleep, focus or calm down, generated live and with the screen off.`

**Descripción larga (es)**

> **SesoLibre Sonidos** genera en vivo el sonido pensado para lo que estás haciendo:
> dormir, concentrarte, meditar, respirar, calmarte o despertar. Suena con la pantalla
> apagada y se detiene sola cuando termina el temporizador.
>
> **Lo comprobado, primero.** Cada sonido dice qué dice la ciencia, con fuentes, y lleva una
> de tres etiquetas: *Comprobado*, *Prometedor* o *Exploración*. Ruido rosa y marrón, agua,
> cuerdas lentas, respiración guiada a seis por minuto.
>
> **Exploración, con etiqueta honesta.** 432 Hz, solfeggio, resonancia Schumann, cimática:
> viven aparte, con la única regla de que no hagan daño.
>
> • Se genera en el teléfono: nunca suena igual y no gasta datos.
> • Controles en la notificación y la pantalla de bloqueo, +15 min y modo noche.
> • Diseñador: arma tus propios sonidos y compártelos como un código de texto.
> • Laboratorio: cimática con la cámara y un experimento de plantas con diario.
> • Sin cuentas, sin anuncios, sin recopilar datos. Español e inglés.
>
> No sustituye atención médica.

**Long description (en)**

> **SesoLibre Sonidos** generates, live, the sound meant for what you are doing: sleeping,
> focusing, meditating, breathing, calming down or waking up. It plays with the screen off
> and stops by itself when the timer ends.
>
> **What's proven comes first.** Every sound says what science says, with sources, and
> carries one of three labels: *Proven*, *Promising* or *Exploration*. Pink and brown noise,
> water, slow strings, guided breathing at six per minute.
>
> **Exploration, honestly labeled.** 432 Hz, solfeggio, Schumann resonance, cymatics: they
> live apart, with the single rule that they do no harm.
>
> • Generated on your phone: never sounds the same and uses no data.
> • Controls in the notification and lock screen, +15 min and night mode.
> • Designer: build your own sounds and share them as a text code.
> • Lab: cymatics with the camera and a plant experiment with a diary.
> • No accounts, no ads, no data collection. Spanish and English.
>
> Not a substitute for medical care.
