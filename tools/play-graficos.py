#!/usr/bin/env python3
"""Gráficos de la ficha de Google Play: ícono 512×512 y gráfico de funciones 1024×500 (es y en).

Usa el ícono de assets/icono/icono-1024.png y la paleta de la app (app/src/index.css).
Salidas en assets/play/. Las capturas (captura-*.png) se toman del teléfono con adb.
Uso: python3 tools/play-graficos.py
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

RAIZ = Path(__file__).resolve().parent.parent
SALIDA = RAIZ / 'assets/play'
FUENTE = '/usr/share/fonts/truetype/noto/NotoSans-{}.ttf'
FONDO, TEXTO = (11, 16, 32), (226, 231, 245)
TENUE = (148, 163, 184)
ETIQUETAS = [(94, 214, 176), (240, 190, 100), (176, 150, 255)]  # comprobado, prometedor, exploración
TEXTOS = {
    'es': ('Sonido para una situación', 'Dormir · Concentrarte · Meditar · Respirar · Calmarte',
           ['Comprobado', 'Prometedor', 'Exploración']),
    'en': ('Sound for a situation', 'Sleep · Focus · Meditate · Breathe · Calm down',
           ['Proven', 'Promising', 'Exploration']),
}


def fuente(peso: str, tam: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(FUENTE.format(peso), tam)


def grafico(idioma: str) -> Image.Image:
    lema, situaciones, etiquetas = TEXTOS[idioma]
    w, h = 1024, 500
    im = Image.new('RGB', (w, h), FONDO)
    d = ImageDraw.Draw(im)
    # Un resplandor suave detrás del ícono.
    for r in range(260, 0, -4):
        a = int(26 * (1 - r / 260))
        d.ellipse((190 - r, 250 - r, 190 + r, 250 + r), fill=tuple(c + a for c in FONDO))
    icono = Image.open(RAIZ / 'assets/icono/icono-1024.png').convert('RGBA').resize((280, 280), Image.LANCZOS)
    im.paste(icono, (50, 110), icono)
    x = 380
    d.text((x, 118), 'SesoLibre Sonidos', font=fuente('Bold', 54), fill=TEXTO)
    d.text((x, 192), lema, font=fuente('Regular', 34), fill=TEXTO)
    d.text((x, 252), situaciones, font=fuente('Regular', 21), fill=TENUE)
    px = x
    for nombre, color in zip(etiquetas, ETIQUETAS):
        f = fuente('Bold', 22)
        ancho = d.textlength(nombre, font=f) + 36
        d.rounded_rectangle((px, 316, px + ancho, 362), radius=23, outline=color, width=2)
        d.text((px + 18, 324), nombre, font=f, fill=color)
        px += ancho + 14
    return im


SALIDA.mkdir(parents=True, exist_ok=True)
Image.open(RAIZ / 'assets/icono/icono-1024.png').convert('RGBA').resize((512, 512), Image.LANCZOS).save(SALIDA / 'icono-512.png')
grafico('es').save(SALIDA / 'grafico-funciones.png')
grafico('en').save(SALIDA / 'grafico-funciones-en.png')
print('listo:', *sorted(p.name for p in SALIDA.iterdir()))
