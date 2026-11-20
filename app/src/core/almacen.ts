// Lo que el laboratorio guarda en el dispositivo: fotos de cimática y experimentos de
// plantas con su diario. IndexedDB, porque las fotos no caben en localStorage. Nada sale
// del teléfono si el usuario no lo comparte.

export interface FotoCimatica {
  id: string;
  fecha: number;
  hz: number;
  /** JPEG como data URL. */
  imagen: string;
}

export interface Medicion {
  fecha: number;
  /** Altura en cm; null si ese día no se midió. */
  control: number | null;
  sonido: number | null;
  nota: string;
  fotoControl?: string;
  fotoSonido?: string;
}

export interface Experimento {
  id: string;
  nombre: string;
  inicio: number;
  recetaId: string;
  minutosDia: number;
  mediciones: Medicion[];
}

const BASE = 'sonidos-lab';
type Tienda = 'cimatica' | 'experimentos';

function abrir(): Promise<IDBDatabase> {
  return new Promise((ok, mal) => {
    const p = indexedDB.open(BASE, 1);
    p.onupgradeneeded = () => {
      p.result.createObjectStore('cimatica', { keyPath: 'id' });
      p.result.createObjectStore('experimentos', { keyPath: 'id' });
    };
    p.onsuccess = () => ok(p.result);
    p.onerror = () => mal(p.error);
  });
}

async function operar<T>(tienda: Tienda, modo: IDBTransactionMode, f: (t: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const db = await abrir();
  return new Promise((ok, mal) => {
    const tx = db.transaction(tienda, modo);
    const r = f(tx.objectStore(tienda));
    tx.oncomplete = () => ok(r.result);
    tx.onerror = () => mal(tx.error);
  });
}

export const leerTodo = <T>(tienda: Tienda) => operar<T[]>(tienda, 'readonly', (t) => t.getAll() as IDBRequest<T[]>);
export const guardar = (tienda: Tienda, valor: FotoCimatica | Experimento) => operar(tienda, 'readwrite', (t) => t.put(valor));
export const borrar = (tienda: Tienda, id: string) => operar(tienda, 'readwrite', (t) => t.delete(id));

export const idNuevo = () => `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 6)}`;

/** El diario como CSV, para abrirlo en una hoja de cálculo. */
export function aCsv(e: Experimento): string {
  const filas = e.mediciones
    .slice()
    .sort((a, b) => a.fecha - b.fecha)
    .map((m) =>
      [
        new Date(m.fecha).toISOString().slice(0, 10),
        Math.round((m.fecha - e.inicio) / 86_400_000),
        m.control ?? '',
        m.sonido ?? '',
        `"${m.nota.replace(/"/g, '""')}"`,
      ].join(','),
    );
  return ['fecha,dia,control_cm,sonido_cm,nota', ...filas].join('\n');
}

/** Diferencia de crecimiento entre la primera y la última medición completas, en cm. */
export function resumen(e: Experimento): { control: number; sonido: number; dias: number } | null {
  const completas = e.mediciones.filter((m) => m.control !== null && m.sonido !== null).sort((a, b) => a.fecha - b.fecha);
  if (completas.length < 2) return null;
  const [a, b] = [completas[0], completas[completas.length - 1]];
  return {
    control: b.control! - a.control!,
    sonido: b.sonido! - a.sonido!,
    dias: Math.round((b.fecha - a.fecha) / 86_400_000),
  };
}
