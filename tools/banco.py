#!/usr/bin/env python3
"""Genera assets/audio/banco.json: el índice de muestras que lee el núcleo.

Los nombres (arpa, cuerdas, hidrofono…) son los que usan las recetas; los archivos son
relativos a assets/audio/. Determinista. Uso: python3 tools/banco.py
"""
import json
import os

AUDIO = os.path.join(os.path.dirname(__file__), "..", "assets", "audio")


def notas(carpeta, desde=0, hasta=127):
    d = os.path.join(AUDIO, "instruments", carpeta)
    out = {}
    for f in sorted(os.listdir(d), key=lambda x: int(x.split(".")[0])):
        n = int(f.split(".")[0])
        if desde <= n <= hasta:
            out[str(n)] = f"instruments/{carpeta}/{f}"
    return out


# sostenido = la nota se mantiene (cuerdas, flauta); si no, se pulsa y decae (arpa, vibráfono).
instrumentos = {
    "arpa": {"sostenido": False, "notas": notas("harp")},
    "vibrafono": {"sostenido": False, "notas": notas("vibes")},
    "marimba": {"sostenido": False, "notas": notas("marimba")},
    "armonicos": {"sostenido": True, "notas": notas("violin_harm")},
    "flauta": {"sostenido": True, "notas": notas("flute_sus")},
    "corno": {"sostenido": True, "notas": notas("horn_sus")},
    "trompeta": {"sostenido": True, "notas": notas("trumpet_sus")},
    "trombones": {"sostenido": True, "notas": notas("trombone_sus")},
    "fagot": {"sostenido": False, "notas": notas("bassoon")},
    "campanas": {"sostenido": False, "notas": notas("tubular_bells")},
    "cello": {"sostenido": True, "notas": notas("cello_sus")},
    "viola": {"sostenido": True, "notas": notas("viola_sus")},
    "violin": {"sostenido": True, "notas": notas("violin_sus")},
    # Un solo "instrumento" de cuerdas: cada registro con su sección.
    "cuerdas": {
        "sostenido": True,
        "notas": {**notas("cello_sus", 0, 47), **notas("viola_sus", 48, 59), **notas("violin_sus", 60, 127)},
    },
}
ambientes = {
    "hidrofono": "ambience/hydrophone_flow.ogg",
    "burbujas": "ambience/bubble_flow.ogg",
    "olas": "ambience/waves_rocks.ogg",
    "chotacabras": "ambience/nightjar.ogg",
    "ranas": "ambience/tree_frog.ogg",
    "grillos": "ambience/mole_cricket.ogg",
    "pita": "ambience/pitta.ogg",
    "gaviotas": "ambience/gulls.ogg",
    "mirlo": "ambience/blackbird.ogg",
    "verderon": "ambience/greenfinch.ogg",
    "mosquitero": "ambience/willow_warbler.ogg",
    "campanero": "ambience/bellbird.ogg",
}
# Cantos de animales: no se tocan a otra velocidad (sonarían a otra especie).
fieles = ["chotacabras", "ranas", "grillos", "pita", "gaviotas", "mirlo", "verderon", "mosquitero", "campanero"]


def eventos(carpeta):
    d = os.path.join(AUDIO, "events", carpeta)
    return [f"events/{carpeta}/{f}" for f in sorted(os.listdir(d))]


golpes = {"gong": eventos("gong"), "platillo": eventos("bowed_cymbal"), "carillon": eventos("windchimes")}
with open(os.path.join(AUDIO, "banco.json"), "w", encoding="utf-8") as f:
    json.dump({"version": 1, "instrumentos": instrumentos, "ambientes": ambientes, "ambientes_fieles": fieles, "eventos": golpes}, f, ensure_ascii=False, indent=1)
    f.write("\n")
print("eventos:", ", ".join(f"{k} ({len(v)})" for k, v in golpes.items()))
print("banco.json:", ", ".join(f"{k} ({len(v['notas'])})" for k, v in instrumentos.items()), "|", ", ".join(ambientes))
