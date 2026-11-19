import { useEffect, useState } from 'react';

import { EN_REPOSO, Sonidos, type Estado } from './motor';

/**
 * El estado del motor, más un reloj local: el plugin avisa solo cuando algo cambia, y
 * entre avisos los segundos se extrapolan aquí (el sonido no depende de esta pantalla).
 */
export function useMotor(): { estado: Estado; segundos: number; error: string | null; limpiarError: () => void } {
  const [estado, setEstado] = useState<Estado>(EN_REPOSO);
  const [recibido, setRecibido] = useState(() => Date.now());
  const [ahora, setAhora] = useState(() => Date.now());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let vivo = true;
    const poner = (e: Estado) => {
      if (!vivo) return;
      setEstado(e);
      setRecibido(Date.now());
      setAhora(Date.now());
    };
    const a = Sonidos.addListener('estado', poner);
    const b = Sonidos.addListener('error', (e) => vivo && setError(e.mensaje));
    Sonidos.estado().then(poner, () => undefined);
    // Al volver a la app (pantalla encendida de nuevo) se pregunta otra vez.
    const alVolver = () => {
      if (document.visibilityState === 'visible') Sonidos.estado().then(poner, () => undefined);
    };
    document.addEventListener('visibilitychange', alVolver);
    return () => {
      vivo = false;
      document.removeEventListener('visibilitychange', alVolver);
      a.then((h) => h.remove());
      b.then((h) => h.remove());
    };
  }, []);

  useEffect(() => {
    if (!estado.sonando) return;
    const id = setInterval(() => setAhora(Date.now()), 500);
    return () => clearInterval(id);
  }, [estado.sonando]);

  const extra = estado.sonando && !estado.cargando ? (ahora - recibido) / 1000 : 0;
  const segundos = Math.min(estado.duracion || Infinity, estado.segundos + Math.max(0, extra));
  return { estado, segundos, error, limpiarError: () => setError(null) };
}
