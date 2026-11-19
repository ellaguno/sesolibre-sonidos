# Palabras

Lo que la app dice y lo que no dice nunca: en la interfaz, en las recetas, en las fichas de
Play y App Store y en este repositorio. El núcleo (`Receta::validar`) rechaza una receta que
use una palabra prohibida en su nombre, descripción o línea de ciencia. Precedentes: FTC vs.
Lumosity (2016) y la regla ya adoptada en SesoLibre Mind.

## Nunca

cura · curar · sana · sanar · sanación · trata · tratar · tratamiento · terapia ·
terapéutico · medicina · (en inglés) cure · heal · healing · treat · treatment · therapy ·
therapeutic · medicine

Tampoco: nombrar enfermedades como destino de una receta ("para la ansiedad", "contra el
insomnio", "Alzheimer"), ni "frecuencia curativa", ni "medicina del sonido", ni "ondas
cerebrales" como promesa ("sincroniza tus ondas cerebrales").

## Sí

- "puede ayudar a", "la gente lo usa para", "pensado para", "acompaña".
- "en investigación", "evidencia mixta", "sin evidencia", "exploración personal".
- Las tres etiquetas, siempre visibles: **Comprobado** · **Prometedor** · **Exploración**.
- La línea "qué dice la ciencia" en cada receta, con fuentes.
- En Exploración, el verbo **pretende**: "esta receta pretende imitar…", nunca "logra".

## Advertencias (claves en `advertencias` de la receta → texto en la app)

| clave | es | en |
|---|---|---|
| `auriculares` | Necesita auriculares (un tono por oído). | Needs headphones (one tone per ear). |
| `no_conducir` | No lo uses al conducir ni al operar máquinas. | Do not use while driving or operating machinery. |
| `epilepsia` | Si tienes epilepsia o fotosensibilidad, consulta antes. | If you have epilepsy or photosensitivity, ask first. |
| `parar_si_molesta` | Si molesta o da dolor de cabeza, para. | If it bothers you or gives you a headache, stop. |
| (siempre) | No sustituye atención médica. | Not a substitute for medical care. |

## Categoría en tiendas

*Salud y bienestar* / *Estilo de vida*. Nunca *Medicina*.
