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
INSTRUMENTOS = {
    "horn_sus": "SGHorn_sus_*",
    "trumpet_sus": "Sum_SHTrumpet_sus_*",
    "trombone_sus": "7TbnEns_sus_*",
    "bassoon": "PSBassoon_*",
    "tubular_bells": "TB_hit_*",
}
# Grabaciones de campo de xeno-canto (CC-BY 4.0): nombre → (número XC, paso alto en Hz,
# segundos desde, segundos que se usan). El crudo no se versiona: se baja aquí por número.
XENO = {
    "nightjar": ("1000907", 300, 5, 90),
    "tree_frog": ("998076", 200, 5, 75),
    "mole_cricket": ("998077", 300, 2, 58),
    "pitta": ("903851", 400, 2, 80),
    "gulls": ("1165177", 300, 1, 39),
}


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

for nombre, (xc, paso_alto, desde, dura) in XENO.items():
    carpeta = os.path.join(AUDIO, "raw", "xeno-canto")
    os.makedirs(carpeta, exist_ok=True)
    crudo = next(iter(glob.glob(os.path.join(carpeta, f"XC{xc}.*"))), None)
    if crudo is None:
        crudo = os.path.join(carpeta, f"XC{xc}.wav")
        subprocess.run(["curl", "-sS", "-L", "-m", "300", "-A", "sesolibre-sonidos-curation", "-o", crudo, f"https://xeno-canto.org/{xc}/download"], check=True)
    corte = ["-ss", str(desde), "-t", str(dura), "-i", crudo]
    limpio = f"highpass=f={paso_alto},highpass=f={paso_alto}"
    media, pico = medir([*corte, "-af", limpio])
    ganancia = min(-24.0 - media, -1.0 - pico)
    salida = os.path.join(AUDIO, "ambience", f"{nombre}.ogg")
    filtro = f"{limpio},volume={ganancia:.2f}dB,afade=t=in:d=0.5,areverse,afade=t=in:d=0.5,areverse"
    subprocess.run(["ffmpeg", "-v", "error", "-y", *corte, "-af", filtro, "-ar", "48000", "-ac", "2", "-c:a", "libvorbis", "-q:a", "4", salida], check=True)
    print(f"ambience/{nombre}: XC{xc}, {dura} s, {ganancia:+.1f} dB")
