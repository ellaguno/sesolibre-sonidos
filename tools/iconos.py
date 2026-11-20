#!/usr/bin/env python3
"""Compone el ícono de SesoLibre Sonidos en SVG, a partir del grafotipo oficial de
SesoLibre (alas + cabeza, assets/src/grafotipo.path, el mismo que usa sesolibre-games).

Escribe assets/icono/<variante>.svg, más las capas del ícono adaptativo de Android de la
variante elegida (fondo y primer plano). Rasterizar: node tools/iconos.mjs
Uso: python3 tools/iconos.py [variante elegida, por defecto "ecualizador"]
"""
import math
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
TRAZO = (RAIZ / "assets/src/grafotipo.path").read_text().strip()
TRANSFORMA = "matrix(1.3333333,0,0,-1.3333333,0,1056) matrix(0.79271731,0,0,0.79271731,-38.456705,414.6568)"
CAJA = (129.0, 13.0, 284.0, 86.0)  # x, y, ancho, alto del grafotipo en sus unidades

# La paleta de la app: verde agua (comprobado) → azul (propio) → violeta (exploración).
MENTA, AZUL, VIOLETA = "#5ed6b0", "#6eaaff", "#b096ff"


def defs():
    x, y, w, h = CAJA
    return f"""<defs>
  <clipPath id="caja"><rect x="{x}" y="{y}" width="{w}" height="{h}"/></clipPath>
  <mask id="marca" maskUnits="userSpaceOnUse" x="{x}" y="{y}" width="{w}" height="{h}">
    <g clip-path="url(#caja)"><path transform="{TRANSFORMA}" d="{TRAZO}" fill="#fff" stroke="#fff" stroke-width="1.1" stroke-linejoin="round"/></g>
  </mask>
  <linearGradient id="color" gradientUnits="userSpaceOnUse" x1="180" y1="0" x2="844" y2="0">
    <stop offset="0" stop-color="{MENTA}"/><stop offset="0.5" stop-color="{AZUL}"/><stop offset="1" stop-color="{VIOLETA}"/>
  </linearGradient>
  <radialGradient id="fondo" cx="0.5" cy="0.36" r="0.8">
    <stop offset="0" stop-color="#263468"/><stop offset="0.55" stop-color="#141b30"/><stop offset="1" stop-color="#0b1020"/>
  </radialGradient>
  <radialGradient id="halo" cx="0.5" cy="0.5" r="0.5">
    <stop offset="0" stop-color="{AZUL}" stop-opacity="0.40"/><stop offset="0.5" stop-color="{VIOLETA}" stop-opacity="0.12"/><stop offset="1" stop-color="{VIOLETA}" stop-opacity="0"/>
  </radialGradient>
  <filter id="brillo" x="-30%" y="-60%" width="160%" height="220%">
    <feGaussianBlur stdDeviation="12" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
  </filter>
</defs>"""


def marca(cx, cy, ancho):
    """El grafotipo centrado en (cx, cy). El degradado va en unidades del lienzo."""
    x, y, w, h = CAJA
    s = ancho / w
    tx, ty = cx - (x + w / 2) * s, cy - (y + h / 2) * s
    # Se pinta con un rect grande del lienzo, enmascarado por la marca ya colocada.
    return (
        f'<mask id="m{int(cx)}{int(cy)}" maskUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024">'
        f'<g transform="translate({tx:.3f},{ty:.3f}) scale({s:.5f})"><rect x="{x}" y="{y}" width="{w}" height="{h}" fill="#fff" mask="url(#marca)"/></g></mask>'
        f'<rect width="1024" height="1024" fill="url(#color)" mask="url(#m{int(cx)}{int(cy)})" filter="url(#brillo)"/>'
    )


def ecualizador(cy=690, n=9, paso=58, ancho=34):
    """Barras redondeadas con forma de campana: el espectro de un sonido suave."""
    alturas = [0.30, 0.50, 0.74, 0.92, 1.0, 0.92, 0.74, 0.50, 0.30]
    x0 = 512 - (n - 1) * paso / 2
    out = []
    for i, a in enumerate(alturas):
        h = 190 * a
        out.append(f'<rect x="{x0 + i * paso - ancho / 2:.1f}" y="{cy - h / 2:.1f}" width="{ancho}" height="{h:.1f}" rx="{ancho / 2}" fill="url(#color)"/>')
    return f'<g filter="url(#brillo)">{"".join(out)}</g>'


def onda(cy=690, amplitud=70, ciclos=2.5):
    """Una sola onda que nace, crece y se apaga."""
    pts = []
    for i in range(0, 201):
        u = i / 200
        x = 200 + u * 624
        env = math.sin(math.pi * u) ** 1.4
        pts.append(f"{x:.1f},{cy - amplitud * env * math.sin(2 * math.pi * ciclos * u):.1f}")
    return f'<polyline points="{" ".join(pts)}" fill="none" stroke="url(#color)" stroke-width="26" stroke-linecap="round" stroke-linejoin="round" filter="url(#brillo)"/>'


def anillos(cx=512, cy=500):
    """Ondas que salen de la cabeza, como en Somnia."""
    return "".join(
        f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="url(#color)" stroke-width="{w}" opacity="{o}"/>'
        for r, w, o in [(250, 12, 0.55), (320, 9, 0.34), (390, 7, 0.18)]
    )


def contenido(variante):
    if variante == "ecualizador":
        return marca(512, 400, 690) + ecualizador()
    if variante == "onda":
        return marca(512, 410, 690) + onda()
    if variante == "anillos":
        return anillos() + f'<circle cx="512" cy="500" r="212" fill="#141b30"/>' + marca(512, 500, 560)
    raise SystemExit(f"variante desconocida: {variante}")


def svg(cuerpo, fondo=True, redondo=True):
    base = ""
    if fondo:
        rx = 'rx="230"' if redondo else ""
        base = f'<rect width="1024" height="1024" {rx} fill="url(#fondo)"/><ellipse cx="512" cy="450" rx="470" ry="330" fill="url(#halo)"/>'
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">{defs()}{base}{cuerpo}</svg>\n'


salida = RAIZ / "assets/icono"
salida.mkdir(parents=True, exist_ok=True)
for v in ("ecualizador", "onda", "anillos"):
    (salida / f"{v}.svg").write_text(svg(contenido(v)))
elegida = sys.argv[1] if len(sys.argv) > 1 else "ecualizador"
# Ícono adaptativo de Android: el sistema recorta; lo importante cabe en el 66 % central.
(salida / "fondo.svg").write_text(svg("", redondo=False))
(salida / "primer-plano.svg").write_text(svg(f'<g transform="translate(512 512) scale(0.8) translate(-512 -512)">{contenido(elegida)}</g>', fondo=False))
(salida / "icono.svg").write_text(svg(contenido(elegida)))
print("assets/icono:", ", ".join(sorted(p.name for p in salida.glob("*.svg"))), "| elegida:", elegida)
