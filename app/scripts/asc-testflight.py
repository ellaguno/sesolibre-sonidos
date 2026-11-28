#!/usr/bin/env python3
"""Pone una compilación en un grupo ABIERTO de TestFlight (enlace público: entra cualquiera
que lo tenga) y la manda a la revisión beta de Apple.

  python3 app/scripts/asc-testflight.py <bundle-id> <número de compilación> [grupo] [notas]

Pasos, todos idempotentes:
  1. espera a que Apple termine de procesar la compilación recién subida;
  2. crea el grupo externo si no existe;
  3. le añade la compilación y escribe las notas de "qué probar";
  4. la envía a revisión beta (obligatoria para testers externos; suele tardar un día);
  5. activa el enlace público e imprime la dirección.

Los pasos 3 a 5 avisan pero no hacen fallar el flujo: la compilación ya está en TestFlight.
La revisión beta exige que en App Store Connect → TestFlight → Información de la prueba
estén la descripción, el correo de comentarios y el contacto; eso se llena una vez a mano.
Credenciales: las de asc.py (ASC_KEY_ID, ASC_ISSUER_ID, ASC_PRIVATE_KEY).
"""
from __future__ import annotations

import os
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import asc  # noqa: E402


def intentar(metodo: str, ruta: str, cuerpo: dict | None = None) -> dict | None:
    """Como asc.llamar, pero un error de la API es un aviso, no el final."""
    try:
        return asc.llamar(metodo, ruta, cuerpo)
    except SystemExit as e:
        print(f'::warning::{e}')
        return None


def main() -> None:
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    bundle, numero = sys.argv[1], sys.argv[2]
    nombre = sys.argv[3] if len(sys.argv) > 3 else 'Abierto'
    notas = Path(sys.argv[4]).read_text(encoding='utf-8') if len(sys.argv) > 4 else ''

    apps = asc.llamar('GET', f'/v1/apps?filter[bundleId]={bundle}')['data']
    if not apps:
        sys.exit(f'✗ no hay app con el bundle {bundle} en App Store Connect')
    app = apps[0]['id']

    # 1. La compilación tarda unos minutos en aparecer y en quedar VALID.
    build = None
    for _ in range(90):
        encontradas = asc.llamar('GET', f'/v1/builds?filter[app]={app}&filter[version]={numero}&limit=5')['data']
        if encontradas:
            estado = encontradas[0]['attributes']['processingState']
            if estado == 'VALID':
                build = encontradas[0]['id']
                break
            if estado in ('FAILED', 'INVALID'):
                sys.exit(f'✗ Apple rechazó la compilación {numero} al procesarla ({estado})')
            print(f'compilación {numero}: {estado}…')
        else:
            print(f'compilación {numero}: aún no aparece…')
        time.sleep(20)
    if not build:
        sys.exit(f'✗ la compilación {numero} no terminó de procesarse en 30 min; vuelve a lanzar este paso')
    print(f'compilación {numero} lista ({build})')

    # 2. El grupo externo.
    grupos = asc.llamar('GET', f'/v1/apps/{app}/betaGroups?limit=200')['data']
    grupo = next((g for g in grupos if g['attributes']['name'] == nombre), None)
    if grupo is None:
        grupo = asc.llamar('POST', '/v1/betaGroups', {'data': {
            'type': 'betaGroups',
            'attributes': {'name': nombre, 'feedbackEnabled': True},
            'relationships': {'app': {'data': {'type': 'apps', 'id': app}}},
        }})['data']
        print(f'grupo «{nombre}» creado')
    gid = grupo['id']
    if grupo['attributes'].get('isInternalGroup'):
        sys.exit(f'✗ «{nombre}» es un grupo interno: no admite enlace público. Usa otro nombre.')

    # 3. La compilación en el grupo, con sus notas.
    if intentar('POST', f'/v1/betaGroups/{gid}/relationships/builds', {'data': [{'type': 'builds', 'id': build}]}) is not None:
        print('compilación añadida al grupo')
    # Apple crea sola la nota del idioma principal: la que ya existe se actualiza.
    puestas = {l['attributes']['locale']: l['id']
               for l in (intentar('GET', f'/v1/builds/{build}/betaBuildLocalizations?limit=50') or {}).get('data', [])}
    for linea in notas.splitlines():
        idioma, _, texto = linea.partition(':')
        locale = {'es': 'es-MX', 'en': 'en-US'}.get(idioma.strip())
        if not locale or not texto.strip():
            continue
        if locale in puestas:
            intentar('PATCH', f'/v1/betaBuildLocalizations/{puestas[locale]}', {'data': {
                'type': 'betaBuildLocalizations', 'id': puestas[locale],
                'attributes': {'whatsNew': texto.strip()[:4000]}}})
        else:
            intentar('POST', '/v1/betaBuildLocalizations', {'data': {
                'type': 'betaBuildLocalizations',
                'attributes': {'locale': locale, 'whatsNew': texto.strip()[:4000]},
                'relationships': {'build': {'data': {'type': 'builds', 'id': build}}},
            }})

    # La descripción beta de la app (Apple la exige para testers externos). Sale de
    # docs/testflight.txt; el correo de comentarios, si se da, de TESTFLIGHT_CORREO.
    descripciones = Path(__file__).resolve().parents[2] / 'docs/testflight.txt'
    if descripciones.exists():
        hechas = {l['attributes']['locale']: l for l in asc.llamar('GET', f'/v1/apps/{app}/betaAppLocalizations?limit=50')['data']}
        correo = os.environ.get('TESTFLIGHT_CORREO') or None
        for linea in descripciones.read_text(encoding='utf-8').splitlines():
            idioma, _, texto = linea.partition(':')
            locale = {'es': 'es-MX', 'en': 'en-US'}.get(idioma.strip())
            if not locale or not texto.strip():
                continue
            attrs = {'description': texto.strip()[:4000], **({'feedbackEmail': correo} if correo else {})}
            if locale in hechas:
                intentar('PATCH', f"/v1/betaAppLocalizations/{hechas[locale]['id']}", {'data': {
                    'type': 'betaAppLocalizations', 'id': hechas[locale]['id'], 'attributes': attrs}})
            else:
                intentar('POST', '/v1/betaAppLocalizations', {'data': {
                    'type': 'betaAppLocalizations', 'attributes': {'locale': locale, **attrs},
                    'relationships': {'app': {'data': {'type': 'apps', 'id': app}}}}})
        print('descripción beta puesta')

    # 4. Revisión beta.
    if intentar('POST', '/v1/betaAppReviewSubmissions', {'data': {
        'type': 'betaAppReviewSubmissions',
        'relationships': {'build': {'data': {'type': 'builds', 'id': build}}},
    }}) is not None:
        print('enviada a revisión beta')
    else:
        print('::warning::no se pudo enviar a revisión beta (¿ya está en revisión, o falta la '
              '"Información de la prueba" en App Store Connect → TestFlight?)')

    # 5. El enlace público.
    intentar('PATCH', f'/v1/betaGroups/{gid}', {'data': {
        'type': 'betaGroups', 'id': gid,
        'attributes': {'publicLinkEnabled': True, 'publicLinkLimitEnabled': False},
    }})
    attrs = asc.llamar('GET', f'/v1/betaGroups/{gid}')['data']['attributes']
    enlace = attrs.get('publicLink')
    if enlace:
        print(f'enlace público: {enlace}')
        salida = os.environ.get('GITHUB_STEP_SUMMARY')
        if salida:
            with open(salida, 'a', encoding='utf-8') as f:
                f.write(f'### TestFlight abierto\n\nGrupo «{nombre}»: {enlace}\n\nFunciona cuando Apple apruebe la revisión beta.\n')
    else:
        print('::warning::el grupo aún no tiene enlace público; actívalo en TestFlight → el grupo → Enlace público')


if __name__ == '__main__':
    main()
