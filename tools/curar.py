#!/usr/bin/env python3
"""Cura el material crudo (assets/audio/raw) → notas que lee el núcleo.

Mismo criterio que sesolibre-lienzo/tools/curate_samples.py, pero solo con ffmpeg:
48 kHz estéreo, sin silencio inicial, RMS -20 dB (pico ≤ -1 dB), cola de 50 ms, Ogg Vorbis q5.
Salida: assets/audio/instruments/<carpeta>/<nota MIDI>.ogg. Determinista.
Uso: python3 tools/curar.py && python3 tools/banco.py
"""
import glob
import os
import re
import subprocess

AUDIO = os.path.join(os.path.dirname(__file__), "..", "assets", "audio")
NOTA = re.compile(r"_([A-G]#?)(\d)(?=[_.])")
NOMBRES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
# carpeta de salida → patrón dentro de raw/vsco-256
INSTRUMENTOS = {"horn_sus": "SGHorn_sus_*"}


def midi(nombre):
    m = NOTA.search(nombre)
    return NOMBRES.index(m.group(1)) + 12 * (int(m.group(2)) + 1)


def medir(args):
    err = subprocess.run(["ffmpeg", "-hide_banner", *args, "-af", "volumedetect", "-f", "null", "-"], capture_output=True, text=True).stderr
    return float(re.search(r"mean_volume: (-?[\d.]+)", err).group(1)), float(re.search(r"max_volume: (-?[\d.]+)", err).group(1))


for carpeta, patron in INSTRUMENTOS.items():
    notas = []
    for f in sorted(glob.glob(os.path.join(AUDIO, "raw", "vsco-256", patron))):
        recorte = "silenceremove=start_periods=1:start_threshold=-48dB:start_silence=0.003"
        media, pico = medir(["-i", f, "-af", recorte])
        ganancia = min(-20.0 - media, -1.0 - pico)
        dur = float(subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", f], capture_output=True, text=True).stdout)
        salida = os.path.join(AUDIO, "instruments", carpeta, f"{midi(os.path.basename(f))}.ogg")
        os.makedirs(os.path.dirname(salida), exist_ok=True)
        filtro = f"{recorte},volume={ganancia:.2f}dB,areverse,afade=t=in:d=0.05,areverse"
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", f, "-af", filtro, "-ar", "48000", "-ac", "2", "-c:a", "libvorbis", "-q:a", "5", salida], check=True)
        notas.append(midi(os.path.basename(f)))
    print(f"{carpeta}: {len(notas)} notas {sorted(notas)}")
