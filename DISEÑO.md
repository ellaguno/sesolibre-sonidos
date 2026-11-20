# Sonidos (sesolibre-sonidos) — diseño

*Versión 1 · 19 de noviembre de 2026*

> **Sonido para una situación, no una frecuencia por fe.** Eliges qué quieres (dormir,
> concentrarte, meditar, respirar, calmarte) y la app genera en vivo el paisaje sonoro que
> más probabilidades tiene de ayudar, con la pantalla apagada y toda la noche si hace falta.
> Lo que la ciencia respalda va primero y sin letra chica; lo alternativo (solfeggio, 432 Hz,
> cimática, agua, plantas) vive en la misma app, en una sala aparte llamada **Exploración**,
> con una etiqueta honesta y la única regla de que no haga daño.

Nació como propuesta en `sesolibre-lienzo/propuestas/onda/` el 19 de noviembre de 2026; ese
mismo día Eduardo aceptó el plan, fijó el nombre **Sonidos** y pidió incluir los sonidos "tipo
psicodélico" (§2.3). Toma de Lienzo la regla de trabajo (se escucha antes de seguir), el manifiesto de
licencias y la idea de un *core* sin interfaz; toma de Mind el stack de interfaz, la regla
"sin promesas médicas" y el determinismo por semilla.

## Estado

- **Fase 0 — Escuchar:** hecha en código (19 nov 2026). Falta el veredicto de Eduardo sobre
  el sonido.
- **Fase 1 — Pantalla apagada:** construida (20 nov 2026), **sin probar en un teléfono**.
  El núcleo usa ya las muestras reales de Lienzo (cuerdas, arpa, vibráfono, armónicos, agua)
  con reverberación; hay ocho recetas; el puente JNI pasa su prueba en la JVM de escritorio;
  la app (Capacitor + React) y el plugin (`MediaSessionService` + `AudioTrack`) compilan a
  un APK de depuración y pasan lint. Lo que solo se puede comprobar en el Pixel: que suene,
  que sobreviva una noche con la pantalla apagada, llamadas, auriculares que se desconectan,
  y cuánta batería gasta.
- **Añadido el 20 nov 2026, a petición de Eduardo tras probar el APK** (compila y pasa
  lint; sin probar en teléfono):
  - Recetas **432 Hz** y **Schumann 7,83 Hz** (solo sus armónicos audibles: la base no se oye).
  - **Naturaleza sintetizada** (`naturaleza`: viento, lluvia, aves) y recetas *Lluvia en la
    ventana* y *Claro del bosque*.
  - **Diseñador**: el usuario elige fondo (mar, olas, burbujas, oleaje, viento, lluvia, aves,
    ruidos), colchón, instrumento, frecuencia y respiración; el resultado es una receta de la
    sala nueva `propio`, validada por el núcleo con los mismos límites.
  - **Guardar y compartir**: los diseños viven en el dispositivo; se comparten como un código
    de texto (`sonidos1:` + receta en base64url) y lo importado entra siempre como `propio`.
  - **Espectrómetro**: FFT en el núcleo (`Motor::espectro`) sobre lo que sale por el altavoz;
    la app lo dibuja solo con la pantalla a la vista.
  - **Aves reales, pendiente.** Los cantos de Merlin Bird ID son de la Macaulay Library de
    Cornell: tienen derechos, no son dominio público, y una app no puede leer los datos de
    otra. Camino limpio: grabaciones CC0 o CC-BY (xeno-canto filtrando licencia, Freesound),
    registradas en `manifest.toml` y añadidas a `tools/banco.py` como ambientes. Mientras, las
    aves son sintéticas y la app lo dice.
  - **Piano, pendiente**: sigue sin haber muestras versionadas.
- **Fase 3 — Hermanas (20 nov 2026):** el núcleo compila a **WebAssembly**
  (`sonidos/wasm`, ABI de C) y corre en un AudioWorklet: la web ya suena de verdad, con el
  mismo código que Android, y valida recetas con el núcleo real. **Enlaces directos**
  (`sesolibre-sonidos://tocar?receta=…&min=…`). Recetas `sonar.calma-ronquido` (para Somnia)
  y `despertar.amanecer`. Guía en `docs/INTEGRAR.md`. Probado en navegador; en Android
  compila. **Sin hacer:** tocar los repositorios de Somnia y Mind, y publicar el AAR y el
  paquete npm (hoy se copian archivos).
- **Capa `eventos`** (gong, platillo frotado, carillón) y **aves reales** de xeno-canto
  (CC BY 4.0, nueve grabaciones, créditos en la app).
- Cambios respecto al plan: el servicio **sí** sostiene un wake lock parcial mientras suena
  (como hace ExoPlayer; se suelta al pausar o terminar); el audio sale por `AudioTrack` desde
  Kotlin en vez de Oboe (la latencia no importa y es menos código nativo); el puente es JNI a
  mano, no uniffi (se revisa en la Fase 3, cuando haga falta Swift). El piano Salamander no
  está versionado en Lienzo: se usan arpa y vibráfono; el piano queda pendiente.

---

## 1. Decisiones tomadas

| Tema | Propuesta |
|---|---|
| Qué es | Un **generador de paisajes sonoros con propósito**: ruido, tonos, pulsos, colchones armónicos, guías de respiración y ambientes, combinados en **recetas** por situación. |
| Qué no es | Un reproductor de MP3 de "frecuencias curativas" ni un dispositivo médico. No diagnostica, no trata, no promete. |
| Orden | **Tres salas**: *Comprobado* (evidencia moderada o mejor), *Prometedor* (evidencia mixta o preliminar) y *Exploración* (sin evidencia, inofensivo). Toda receta lleva su etiqueta a la vista y una línea de "qué dice la ciencia". |
| Excluido | Lo que puede hacer daño o sustituir un tratamiento: frecuencias "Rife" contra enfermedades, infrasonidos o ultrasonidos a volumen alto, cualquier receta nombrada como cura. **Única excepción** (decisión de Eduardo): los sonidos que *pretenden* efectos tipo psilocibina entran en Exploración, diciendo claramente qué pretenden y qué se sabe (§2.3). |
| Sonido | **Síntesis en tiempo real** en Rust (determinista por semilla), más bucles cortos de grabaciones de campo CC0/CC-BY. Nada de archivos de una hora. |
| Pantalla apagada | El audio corre en un **servicio nativo en primer plano** (Android `MediaSessionService`, iOS *background audio*), nunca en la vista web ni en la actividad. |
| Interfaz | Capacitor + React (stack de Mind) como **control remoto** del servicio; el core de sonido se expone como plugin nativo. Alternativa evaluada y descartada salvo sorpresa: Compose puro (§6). |
| Integración | El mismo core se entrega como **crate Rust, AAR Android y paquete npm (WASM)**, para que Somnia lo embeba en su propio servicio nocturno y Mind lo use dentro de su WebView. Además, *deep links* para abrir una receta desde fuera. |
| Público | Yo, familia y amigos. Sin cuentas, sin rastreo, sin monetización. |
| Licencias | Código Apache-2.0 (como Mind). Assets solo CC0 / CC-BY / CC-BY-SA, registrados en `manifest.toml` como en Lienzo. |

---

## 2. Análisis: qué se sabe de cada cosa

Resumen de la evidencia a noviembre de 2026, con la sala que le corresponde en la app.
Fuentes en §10. Criterio: revisiones sistemáticas y ensayos controlados por encima de
estudios aislados; la etiqueta se puede mover cuando salga evidencia nueva.

### 2.1 Sala *Comprobado* (evidencia moderada o buena, uso seguro)

| Qué | Para qué | Qué dice la evidencia | Nota de diseño |
|---|---|---|---|
| **Música lenta y relajante** (60–80 BPM, sin letra, dinámica plana) | Dormir, calmar | Cochrane 2022 (Jespersen): *probablemente* mejora la calidad subjetiva del sueño en adultos con insomnio, certeza moderada; sin efectos adversos reportados. | El colchón armónico generativo de la app (y, en el futuro, obras de Lienzo) cumple este papel. |
| **Ruido de enmascaramiento** (blanco, rosa, marrón; filtrado) | Conciliar el sueño con ruido externo; concentrarse en entornos ruidosos | Evidencia modesta y heterogénea, en general positiva para latencia de sueño cuando hay ruido ambiental; la calidad de los estudios es baja pero el mecanismo (enmascarar) es trivial y seguro. | Rosa y marrón por defecto (menos fatigosos que el blanco). Volumen techo bajo para noches enteras. |
| **Respiración guiada por sonido** (≈6 respiraciones/min, exhalación larga) | Calmar, meditar, dormir | La respiración lenta tiene evidencia consistente sobre ansiedad y variabilidad cardiaca; el sonido es solo el metrónomo. | Un *swell* que sube y baja; sin voz. Es probablemente la función más útil de toda la app. |
| **Sonidos de naturaleza** (lluvia, mar, viento, arroyo) | Calmar, dormir, concentrar | Varios ensayos muestran recuperación de estrés más rápida que con ruido urbano o silencio; efectos pequeños-medianos. | Bucles de campo CC0; dos copias a velocidades distintas para que no se note el ciclo (truco de Lienzo). |
| **Temporizador con desvanecimiento** | Dormir | No es "frecuencia", es higiene de sueño: evita escuchar toda la noche a volumen fijo. | Siempre presente; por defecto 30–45 min con *fade* de varios minutos. |

### 2.2 Sala *Prometedor* (evidencia mixta o preliminar, uso seguro)

| Qué | Para qué | Qué dice la evidencia | Nota de diseño |
|---|---|---|---|
| **Pulsos binaurales** (dos tonos, uno por oído, diferencia 1–40 Hz) | Dormir (delta/theta), concentrar (beta), meditar (theta/alfa) | Metaanálisis 2019 (Garcia-Argibay): efecto medio (g≈0,45) sobre ansiedad, cognición y dolor, muy heterogéneo. Revisiones 2023–2025: ansiedad y sueño pequeños-moderados; cognición mixta; varios ensayos nulos. Un estudio 2025: solo *beats + música* bajó la ansiedad. | Siempre dentro de un colchón musical o de ruido, nunca a pelo. Requiere auriculares; la app lo detecta y lo avisa. **Advertencia: no al conducir ni operar máquinas; precaución en epilepsia.** |
| **Tonos isocrónicos** (un tono pulsado) | Igual que los binaurales, sin auriculares | Menos estudiado que los binaurales; mismo supuesto (arrastre de ondas cerebrales). | Opción cuando no hay auriculares. |
| **Estimulación a 40 Hz** (gamma, clics o modulación) | Memoria, atención (y la hipótesis Alzheimer) | Cognito Therapeutics presentó en noviembre 2026 resultados del ensayo pivotal HOPE (673 personas, luz+sonido 1 h/día) con señales de freno a la progresión en Alzheimer leve; pendiente publicación revisada y FDA. En personas sanas no hay evidencia de beneficio. | Se ofrece como sesión de 20–60 min, etiquetado "en investigación"; **sin mención de Alzheimer** en la app, que es territorio médico. |
| **Ruido rosa sincronizado con el sueño lento** | Sueño profundo, memoria | Funciona en laboratorio *en bucle cerrado* (EEG detecta la fase de la onda lenta). Los metaanálisis 2022–2023 muestran efecto decreciente con los años y problemas de ciego. **Sin EEG no hay bucle cerrado**, así que en la app es solo ruido rosa suave. | Honestidad: ruido rosa nocturno sí; "mejora tu memoria mientras duermes", no. |
| **ASMR / sonidos suaves cercanos** | Calmar, dormir | Estudios pequeños con efectos de relajación en quienes responden. | Una familia de texturas (lluvia en ventana, papel, agua corriendo cerca). |

### 2.3 Sala *Exploración* (sin evidencia, inofensivo a volumen seguro)

Aquí entra todo lo "fringe" que pidió Eduardo. La etiqueta fija de la sala: **"Sin evidencia
científica. Esto es exploración personal: prueba y saca tus conclusiones."** Nada en esta
sala usa las palabras curar, sanar, tratar.

| Qué | Qué se afirma | Qué hay | Cómo entra en la app |
|---|---|---|---|
| **Solfeggio** (174, 285, 396, 417, 528, 639, 741, 852, 963 Hz) | 528 Hz "repara el ADN", etc. | Sin estudios serios. Son tonos puros inofensivos. | Un afinador de colchón: el pad se afina a la frecuencia elegida. Suena bien; se dice lo que es. |
| **432 Hz vs 440 Hz** | Más "natural", más relajante | Dos o tres estudios pequeños de baja calidad con efectos menores en frecuencia cardiaca. | Interruptor global de afinación del colchón (432/440). Barato y agradable. |
| **Resonancia Schumann (7,83 Hz)** | Sintoniza con la Tierra | Es real como fenómeno electromagnético; que un *beat* audible de 7,83 Hz haga algo, no se ha demostrado. | Un preset más de pulso binaural/isocrónico en theta. |
| **Chakras / notas** | Cada nota activa un centro | Tradición, no ciencia. | Siete afinaciones del colchón con su color. |
| **Cimática** (figuras de agua y arena bajo vibración) | El agua "recibe" la frecuencia | La **física es real** (ondas estacionarias de Chladni y Faraday); el salto a "agua sanadora" (Emoto) nunca se replicó. | Modo *Laboratorio*: barrido de frecuencias con el altavoz del teléfono sobre un plato con agua, cámara en vivo, guardar foto. Es visualmente hermoso y es ciencia de verdad. |
| **Agua "estructurada" / informada** | Imbuir frecuencias en el agua de riego | Marketing sin validación independiente. Ejemplo: Harvest Harmonics (Kyminasi), transmisores en la tubería con "frecuencias" para absorber agua, nitrógeno y luz; resultados autoinformados y mixtos. | Se puede reproducir la receta (tonos sobre un recipiente); se etiqueta como lo que es. |
| **Sonido para plantas** ("sonic bloom", tecnología acústica agrícola china) | Más crecimiento y absorción de nutrientes | Estudios de baja calidad, algunos positivos, sin replicación limpia; un estudio escolar reciente no halló efecto. | **Experimento con control**: la app propone un protocolo (dos macetas iguales, una con sonido 1 h/día, fotos semanales) y lleva el diario. Ciencia ciudadana en lugar de fe. |
| **Sonidos "tipo psicodélico"** ("drogas digitales", i-Doser y similares) | Pulsos binaurales y texturas que imitan estados alterados, "como psilocibina" | Los estudios sobre "drogas digitales" no encuentran efectos psicodélicos; reportan relajación, sugestión y, en algunas personas, molestia o dolor de cabeza. Es sonido a volumen seguro: no altera la química de nadie. | Situación **Viaje** dentro de Exploración. La ficha dice en mayúsculas que la receta *pretende* imitar un estado alterado y que no hay evidencia de que lo logre. Advertencias: auriculares, no conducir, epilepsia, parar si molesta. Es la única excepción a la regla de "lo que afirma efectos fuertes, fuera": Eduardo decidió incluirlos **declarando la intención**. |

### 2.4 Excluido (puede hacer daño o engañar)

- **Frecuencias "Rife"** y cualquier receta nombrada por enfermedad (cáncer, parásitos,
  covid…). Suplanta tratamiento; FDA y FTC han actuado contra ello.
- **Infrasonidos** (< 20 Hz) y **ultrasonidos** a volumen alto; **tonos agudos fuertes**
  sostenidos (fatiga auditiva, riesgo en tinnitus).
- **Volumen sin techo.** La app limita la salida (limitador + techo configurable, por
  defecto conservador para noches con auriculares; referencia WHO: 80 dB(A) ≈ 40 h/semana).
- **Promesas.** Ni en la ficha de Play/App Store ni dentro de la app: "mejora", "cura",
  "sana", "trata", "terapia" como sustantivo. Se dice "puede ayudar a", "la gente lo usa
  para", "en investigación". Precedente: FTC vs. Lumosity (2016), ya adoptado en Mind.

---

## 3. La experiencia

### 3.1 Entrar

Una pantalla con **situaciones**, no con frecuencias: *Dormir · Soñar en calma · Concentrar
· Meditar · Respirar · Calmar · Despertar* y abajo, separada por una línea y un color
distinto, la sala **Exploración**. Cada situación tiene 2–4 recetas con su etiqueta
(✔ comprobado · ◐ prometedor · ◌ exploración), su duración sugerida y una línea de "qué
dice la ciencia" que se expande a las fuentes.

### 3.2 Sonar

Tocar una receta la hace sonar en 2 segundos con *fade in*. Se puede ajustar en caliente:
intensidad de cada capa (ambiente, tono/pulso, colchón, respiración), afinación, duración,
desvanecimiento final. La pantalla puede apagarse: el sonido sigue, con controles en la
notificación y en la pantalla de bloqueo (play/pausa, +15 min, parar), y botones de los
auriculares.

### 3.3 Sentir, no medir

No hay puntuaciones ni gráficas de "ondas cerebrales". Al terminar una sesión se puede
anotar en una palabra cómo fue (opcional). Nada más. La app no mide nada del cuerpo.

### 3.4 Exploración

Misma mecánica, otro tono visual (más de taller que de dormitorio). Incluye los presets
(solfeggio, 432, chakras, Schumann), el **Laboratorio de cimática** (cámara + barrido de
frecuencias) y el **Experimento de plantas** (protocolo con control y diario de fotos).

---

## 4. Recetas: el formato

Una receta es un JSON pequeño y versionado, igual que una obra de Lienzo. Es el contrato
entre el core y cualquier interfaz (la app, Somnia, Mind):

```json
{
  "version": 1,
  "id": "dormir.rosa-colchon",
  "sala": "comprobado",
  "situacion": "dormir",
  "duracion_s": 2700,
  "fade_in_s": 8,
  "fade_out_s": 300,
  "semilla": 12345,
  "capas": [
    { "tipo": "ruido", "color": "rosa", "paso_bajo_hz": 1800, "nivel_db": -22 },
    { "tipo": "colchon", "afinacion_hz": 440, "modo": "re_pentatonico", "nivel_db": -26, "ciclo_s": 240 },
    { "tipo": "pulso", "forma": "binaural", "portadora_hz": 180, "beat_hz": 2.5, "nivel_db": -30, "requiere_auriculares": true }
  ],
  "techo_db": -14,
  "advertencias": ["auriculares", "no_conducir"]
}
```

Tipos de capa previstos: `ruido` (blanco/rosa/marrón, filtros), `tono` (puro, con
armónicos suaves), `pulso` (binaural, isocrónico, AM a 40 Hz), `colchon` (pad modal
generativo, afinable a 432/440 o a cualquier Hz), `ambiente` (bucle de campo con dos
velocidades), `respiracion` (swell con tiempos de inhalar/retener/exhalar), `barrido`
(solo Exploración: frecuencia que recorre un rango para la cimática).

**Semilla + receta → el mismo sonido, siempre.** Permite rendir WAV fuera de línea para
escucharlo y probarlo sin pantalla, como en Lienzo.

---

## 5. El motor de sonido (`sonidos-core`)

Rust puro, sin UI, sin dependencias de plataforma. Entradas: una receta y una semilla.
Salida: bloques de audio estéreo a 48 kHz o un WAV.

- **Generadores**: ruido con filtros de un polo y biquad; osciladores con tabla de senos
  y suavizado de parámetros (nada hace *clic* al cambiar); pad modal con 4–6 voces
  detonadas lentamente (lo que en Lienzo es el "colchón"; aquí sintetizado para no cargar
  muestras, o con 2–3 muestras CC0 si el sintético suena a plástico; se decide oyendo).
- **Mezcla y seguridad**: niveles en dB por capa, limitador en el master, techo
  configurable, *fades* largos, y una regla dura: **ninguna capa puede subir sola** más de
  X dB en Y segundos (nada de sobresaltos a las 3 a.m.).
- **Reloj**: el core avanza por bloques; el programador de eventos (cambios de acorde del
  colchón, fases de respiración, fin del temporizador) va en tiempo de muestras, no de
  frames.
- **Coste**: todo lo anterior cabe en < 2 % de CPU de un teléfono medio; noche entera sin
  calentar.
- **Interfaces**: `uniffi` genera las ligaduras Kotlin y Swift; `wasm-bindgen` el módulo
  WASM que corre en un `AudioWorklet` para la web y para Mind.
- **CLI**: `onda render receta.json --semilla 7 --minutos 3 salida.wav`. Es la Fase 0.

Qué se reutiliza de Lienzo: la idea del core sin UI y su CLI, el truco de los dos bucles
de ambiente a velocidades distintas, el limitador agresivo, el ciclo de acordes lento sobre
Re pentatónico (`lienzo-core/src/theory.rs` como referencia), el formato de
`manifest.toml` y las grabaciones de agua ya curadas (`assets/audio/`). Qué no: Bevy y
`cpal` dentro de la app (ver §6).

---

## 6. Arquitectura y el problema de la pantalla apagada

Es el requisito que decide la arquitectura, así que va primero.

**Por qué ni Lienzo ni Mind sirven tal cual.** Lienzo corre dentro de una actividad Bevy:
al apagar la pantalla Android la pausa y, en segundo plano sin servicio, la congela o mata.
Mind corre en un WebView de Capacitor: el sistema suspende la vista web poco después de
salir del primer plano, y la Web Audio API no sobrevive a la pantalla apagada ni en Android
ni en iOS. Los plugins de audio de fondo para Capacitor reproducen *archivos*; ninguno
sintetiza.

**La solución en Android** (confirmada en la documentación de Media3): un
`MediaSessionService` propio, declarado con `foregroundServiceType="mediaPlayback"` y los
permisos `FOREGROUND_SERVICE` y `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, que arranca
**mientras la app está visible** (Android 12+ prohíbe arrancarlo desde segundo plano),
pide *audio focus* (Android 15+ lo niega a quien no sea app visible o servicio en primer
plano) y escribe el audio con AAudio/Oboe desde el core en Rust. La `MediaSession` da los
controles de pantalla de bloqueo y de auriculares; la reproducción activa mantiene el
proceso fuera de Doze (con un wake lock parcial mientras suena, por robustez). Para iOS, más adelante: `AVAudioSession`
en categoría `playback` y el *background mode* "audio".

```
sonidos/
├── core/        Rust puro: recetas, generadores, mezcla, limitador, reloj, render WAV. Headless.
├── bindings/    uniffi (Kotlin/Swift) y wasm-bindgen (AudioWorklet). Publica AAR y npm.
├── app/         Capacitor + React (stack de Mind): catálogo, recetas, Exploración, ajustes.
│   └── plugin/  Plugin Capacitor "SonidosEngine": MediaSessionService + AAudio ← core (Android);
│                AVAudioEngine ← core (iOS, después).
├── assets/      bucles de campo CC0/CC-BY, manifest.toml → CRÉDITOS.
└── tools/       curación de bucles, generación de créditos, render de pruebas.
```

**Interfaz: Capacitor (recomendado) o Compose.** La vista web es solo el control remoto:
elige la receta, llama al plugin, y el plugin manda el JSON al servicio. Ventajas: se hereda
de Mind el i18n es/en, ajustes, almacenamiento, tema claro/oscuro, el flujo de Play y la
PWA (la versión web sonará con la pantalla encendida vía WASM, suficiente para probar y
para Mind). Coste: el plugin se escribe en Kotlin de todos modos. La alternativa, una app
Compose pura, da una integración más fina (widgets, Wear) a cambio de duplicar lo que Mind
ya resolvió; se descarta salvo que el plugin dé más guerra de la prevista.

---

## 7. Integración con Somnia y Mind

Tres niveles, de más robusto a más simple. Se recomiendan los dos primeros.

1. **Biblioteca embebida (para Somnia).** Somnia ya corre de noche un servicio en primer
   plano para escuchar ronquidos; es el lugar natural para generar el sonido. Somnia añade
   la dependencia `sonidos-core` (AAR con ligaduras Kotlin), elige una receta por id y la toca
   dentro de **su propio** servicio. Nada de despertar a otra app: Android 12+ restringe
   arrancar servicios en primer plano desde segundo plano y la cadena Somnia → Sonidos a las
   3 a.m. es justo ese caso. Receta propuesta `sonar.calma-ronquido`: colchón muy grave y
   lento + ruido marrón, 8–10 min, *fade* largo, techo bajo. Honestidad en el diseño: no
   hay evidencia de que un sonido reduzca el ronquido; lo que sí existe (terapia posicional)
   usa vibración, no sonido. Se etiqueta ◐ y se mide en casa: Somnia ya cuenta ronquidos,
   así que puede comparar noches con y sin receta. Eso es un experimento real y gratis.
2. **Paquete npm con WASM + AudioWorklet (para Mind).** Mind corre con pantalla encendida,
   así que la Web Audio API basta. Casos: ruido o pulso beta suave durante un puzzle largo,
   respiración de 60 s antes de la sesión diaria, un tono de "foco" al empezar. Mind decide
   si lo expone como ajuste ("sonido de fondo durante los ejercicios").
3. **Deep links y intents (para cualquiera).** `sesolibre-sonidos://tocar?receta=dormir.rosa-colchon&min=30`
   abre la app en primer plano y arranca el servicio. Sirve para Somnia en los casos donde
   el usuario aún está despierto (rutina de acostarse) y para accesos directos y automatismos.

Las tres comparten el **formato de receta** (§4) y el catálogo con sus etiquetas, así
ninguna app hermana tiene que inventar sonido ni volver a decidir qué es honesto decir.

---

## 8. Seguridad, salud y palabras

- **Oído**: techo de salida con limitador; preset "noche con auriculares" más bajo;
  recordatorio de volumen al pasar de 60 min seguidos con auriculares; nada por debajo de
  20 Hz ni por encima de 16 kHz a nivel audible; sin sonidos súbitos.
- **Advertencias por receta**: binaurales e isocrónicos → "no al conducir ni operar
  máquinas; si tienes epilepsia, consulta antes". Todo → "no sustituye atención médica".
  Tinnitus/hiperacusia → recetas con tonos puros marcadas.
- **Palabras**: lista de términos permitidos y prohibidos en `docs/PALABRAS.md`, revisada
  antes de cada publicación (igual que Mind con "edad cerebral"). Ficha de Play y App Store
  en categoría *Salud y bienestar / Estilo de vida*, no *Medicina*.
- **Privacidad**: sin cuentas, sin rastreo, sin micrófono (la cimática usa la cámara solo
  en vivo y guarda la foto si el usuario lo pide). Política como la de Mind.
- **Licencias y créditos**: `manifest.toml` por asset; un asset sin registro no entra al
  build; `CRÉDITOS.md` generado.

---

## 9. Fases

| Fase | Entregable | Pregunta que responde |
|---|---|---|
| **0 · Escuchar** 🔧 *código hecho el 19 nov 2026; falta escuchar* | `sonidos-core` + CLI que rinde 6 recetas a WAV (dormir, concentrar, respirar, meditar, solfeggio 528, viaje). Eduardo las escucha con auriculares y en altavoz. | ¿El sintético suena bien o hacen falta muestras? ¿Los pulsos dentro de música molestan? Si no gusta, se cambia el sonido, no la arquitectura. |
| **1 · Pantalla apagada** 🔧 *construida el 20 nov 2026; falta probar en el Pixel* | App Android mínima (tres situaciones) con el servicio `mediaPlayback`, controles de bloqueo, temporizador y techo de volumen. Una noche entera en el Pixel sin cortes ni batería rara. | ¿Sobrevive al Doze, al cambio de auriculares, a una llamada? Es el riesgo técnico número uno y se resuelve antes de hacer el catálogo. |
| **2 · Catálogo y salas** | Las siete situaciones, etiquetas con fuentes, sala Exploración con presets, ajustes heredados de Mind, es/en. | ¿Se entiende la diferencia entre salas sin leer? ¿Se usa de noche más de una semana? |
| **3 · Hermanas** 🔧 *núcleo WASM, enlaces y guía hechos el 20 nov 2026; falta integrarlo en Somnia y Mind* | AAR + npm publicados; Somnia embebe `sonar.calma-ronquido` y compara noches; Mind ofrece sonido de fondo; deep links. | ¿Somnia nota algo medible? ¿Mind lo quiere? |
| **4 · Laboratorio** | Cimática con cámara y barrido; experimento de plantas con protocolo y diario; sesión 40 Hz larga. | ¿Lo fringe se disfruta como exploración honesta? |
| **5 · Más** | PWA pública, iOS (cuando haya Mac), obras de Lienzo como paisaje para dormir, Wear/widgets. | — |

### Regla de trabajo

La misma que en Lienzo: cada cambio al core produce un WAV que **tú** escuchas antes de
seguir; cada receta nueva se duerme al menos una noche con ella antes de entrar al catálogo;
las métricas (espectro, nivel, CPU, batería) sirven para encontrar bugs, nunca para decidir
si algo ayuda.

---

## 10. Preguntas abiertas

- ~~Nombre~~: **Sonidos** (decidido el 19 nov 2026).
- **¿Capacitor o Compose?** Capacitor por herencia de Mind (aceptado); se confirma en la
  Fase 1 según lo que cueste el plugin.
- **¿Somnia embebe el core o solo lanza deep links?** Embeber (aceptado); depende de en
  qué esté escrita Somnia (si es Capacitor, el AAR entra por un plugin igual que aquí).
- **Techo de volumen por defecto.** Conservador siempre; el número exacto se decide oyendo
  de noche en la Fase 1.
- **Colchón sintético vs. muestras.** Se decide en la Fase 0.
- **Lienzo como sonido para dormir.** Una obra de Lienzo ya es un paisaje sonoro lento,
  determinista y hermoso; exportarla como `ambiente` de Sonidos cerraría el círculo entre las
  dos apps. Está en `IDEAS.md` de Lienzo hasta la Fase 5.

---

## 11. Fuentes consultadas (19 nov 2026)

- Música e insomnio: [Cochrane CD010459, Jespersen et al. 2022](https://www.cochrane.org/CD010459/SLEEP_music-for-insomnia-adults) · [PubMed](https://pubmed.ncbi.nlm.nih.gov/36000763/)
- Pulsos binaurales: [Garcia-Argibay et al., metaanálisis 2019](https://pubmed.ncbi.nlm.nih.gov/30073406/) · [metaanálisis memoria/atención 2023](https://pubmed.ncbi.nlm.nih.gov/35842538/) · [Frontiers in Psychology 2025, beats + música](https://www.frontiersin.org/articles/10.3389/fpsyg.2025.1539823/full) · [revisión sistemática 2024 (MDPI)](https://www.mdpi.com/2850398)
- Estimulación acústica del sueño lento: [Harlow et al. 2023, replicabilidad](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12713896/) · [metaanálisis memoria y arquitectura del sueño](https://aminer.org/pub/61a8a3825244ab9dcb6b184d/the-impact-of-acoustic-stimulation-during-sleep-on-memory-and-sleep-architecture) · [Frontiers in Sleep 2023](https://frontiersin.org/articles/10.3389/frsle.2023.1082253/pdf)
- 40 Hz: [OVERTURE, Frontiers in Neurology 2024](https://www.frontiersin.org/articles/10.3389/fneur.2024.1343588) · [BrightFocus sobre resultados HOPE, nov 2026](https://www.brightfocus.org/news/light-and-sound-therapy-slows-cognitive-decline-in-people-with-mild-alzheimers-disease/) · [NeurologyLive, abr 2025](https://www.neurologylive.com/view/cognito-gamma-sensory-stimulation-leads-preserved-corpus-callosum-structure-alzheimers)
- "Frecuencias" para riego y plantas: [Harvest Harmonics / Kyminasi](https://www.agriculture-xprt.com/products/natural-kyminasi-plant-booster-technology-940524) · [estudio escolar sin efecto](https://www.emerginginvestigators.org/articles/24-179/pdf) · [patente "Plant treatment process"](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/5731265)
- Android en segundo plano: [Media3, reproducción en segundo plano con MediaSessionService](https://developer.android.com/media/media3/session/background-playback?hl=en) · [cambios en servicios en primer plano](https://developer.android.com/develop/background-work/services/fgs/changes?authuser=0)
- Capacitor y audio de fondo: [Capawesome, audio en segundo plano](https://capawesome.io/blog/how-to-play-audio-in-the-background-in-capacitor/) · [Capgo, native-audio](https://capgo.app/docs/plugins/native-audio/getting-started/) · [WebKit: Web Audio en segundo plano en iOS](https://bugs.webkit.org/show_bug.cgi?id=173332)
