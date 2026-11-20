import { Capacitor } from '@capacitor/core';
import { Directory, Filesystem } from '@capacitor/filesystem';
import { Share } from '@capacitor/share';

/** Dibuja la fuente en un lienzo de como mucho `lado` px y devuelve un JPEG (data URL). */
export function aJpeg(fuente: CanvasImageSource, ancho: number, alto: number, lado = 1280, rotulo?: string): string {
  const k = Math.min(1, lado / Math.max(ancho, alto));
  const c = document.createElement('canvas');
  c.width = Math.round(ancho * k);
  c.height = Math.round(alto * k);
  const ctx = c.getContext('2d')!;
  ctx.drawImage(fuente, 0, 0, c.width, c.height);
  if (rotulo) {
    const cuerpo = Math.max(16, Math.round(c.width / 28));
    ctx.font = `600 ${cuerpo}px system-ui, sans-serif`;
    const margen = cuerpo * 0.6;
    const w = ctx.measureText(rotulo).width;
    ctx.fillStyle = 'rgba(11,16,32,0.7)';
    ctx.fillRect(0, c.height - cuerpo - margen * 2, w + margen * 2, cuerpo + margen * 2);
    ctx.fillStyle = '#e2e7f5';
    ctx.fillText(rotulo, margen, c.height - margen - cuerpo * 0.2);
  }
  return c.toDataURL('image/jpeg', 0.85);
}

/** Una foto elegida o tomada con `<input type="file">`, reducida a JPEG. */
export function archivoAJpeg(archivo: File, lado = 1024): Promise<string> {
  return new Promise((ok, mal) => {
    const url = URL.createObjectURL(archivo);
    const img = new Image();
    img.onload = () => {
      URL.revokeObjectURL(url);
      ok(aJpeg(img, img.naturalWidth, img.naturalHeight, lado));
    };
    img.onerror = () => {
      URL.revokeObjectURL(url);
      mal(new Error('no se pudo leer la imagen'));
    };
    img.src = url;
  });
}

/** Saca un archivo del dispositivo: hoja de compartir en Android, descarga en la web. */
export async function exportar(nombre: string, contenido: string, tipo: 'jpeg' | 'csv'): Promise<void> {
  if (Capacitor.isNativePlatform()) {
    const esFoto = tipo === 'jpeg';
    const { uri } = await Filesystem.writeFile({
      path: nombre,
      data: esFoto ? contenido.split(',')[1] : btoa(unescape(encodeURIComponent(contenido))),
      directory: Directory.Cache,
    });
    await Share.share({ title: nombre, files: [uri] });
    return;
  }
  const a = document.createElement('a');
  a.href = tipo === 'jpeg' ? contenido : URL.createObjectURL(new Blob([contenido], { type: 'text/csv' }));
  a.download = nombre;
  a.click();
}
