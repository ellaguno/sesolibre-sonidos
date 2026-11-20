import { WebPlugin } from '@capacitor/core';

import { EN_REPOSO, type Estado, type OpcionesTocar, type SonidosPlugin } from './motor';

// El motor en el navegador: el mismo núcleo que en Android, compilado a WebAssembly
// (sonidos/wasm → public/motor/sonidos.wasm) y corriendo en un AudioWorklet. Suena mientras
// la pestaña esté viva; con la pantalla apagada manda el sistema, no nosotros: para eso
// está la app de Android.

const BASE = import.meta.env.BASE_URL;
const cod = new TextEncoder();
const dec = new TextDecoder();

type Exportes = {
  memory: WebAssembly.Memory;
  reservar(n: number): number;
  liberar(p: number, n: number): void;
  texto_ptr(): number;
  texto_len(): number;
  validar(p: number, n: number): number;
  archivos(p: number, n: number): number;
};

type Respuesta = { segundos: number; duracion?: number; bandas?: Float32Array | null };

export class SonidosWeb extends WebPlugin implements Omit<SonidosPlugin, 'addListener'> {
  private e: Estado = { ...EN_REPOSO, auriculares: true };
  private bytes: Promise<ArrayBuffer> | null = null;
  private local: Promise<Exportes> | null = null;
  private ctx: AudioContext | null = null;
  private nodo: AudioWorkletNode | null = null;
  private cargadas = new Set<string>();
  private esperas = new Map<number, (r: Respuesta) => void>();
  private alCrear: ((m: { ok: boolean; texto: Uint8Array | null }) => void) | null = null;
  private turno = 0;
  private volumenDb = 0;
  private suspension: ReturnType<typeof setTimeout> | null = null;

  private wasm(): Promise<ArrayBuffer> {
    return (this.bytes ??= fetch(`${BASE}motor/sonidos.wasm`).then((r) => {
      if (!r.ok) throw new Error('falta motor/sonidos.wasm (npm run wasm)');
      return r.arrayBuffer();
    }));
  }

  /** Una copia del núcleo en el hilo principal, solo para validar y preguntar archivos. */
  private nucleo(): Promise<Exportes> {
    return (this.local ??= this.wasm().then(async (b) => (await WebAssembly.instantiate(b, {})).instance.exports as unknown as Exportes));
  }

  private async conReceta<T>(receta: string, f: (x: Exportes, p: number, n: number) => T): Promise<T> {
    const x = await this.nucleo();
    const b = cod.encode(receta);
    const p = x.reservar(b.length);
    new Uint8Array(x.memory.buffer, p, b.length).set(b);
    try {
      return f(x, p, b.length);
    } finally {
      x.liberar(p, b.length);
    }
  }

  private static texto(x: Exportes): string {
    return dec.decode(new Uint8Array(x.memory.buffer, x.texto_ptr(), x.texto_len()));
  }

  async validar(o: { receta: string }): Promise<{ motivo: string | null }> {
    return this.conReceta(o.receta, (x, p, n) => ({ motivo: x.validar(p, n) ? null : SonidosWeb.texto(x) }));
  }

  private async audio(): Promise<AudioWorkletNode> {
    if (this.nodo && this.ctx) return this.nodo;
    const ctx = new AudioContext({ sampleRate: 48_000, latencyHint: 'playback' });
    await ctx.audioWorklet.addModule(`${BASE}motor/worklet.js`);
    const nodo = new AudioWorkletNode(ctx, 'sonidos', { numberOfInputs: 0, outputChannelCount: [2] });
    nodo.port.onmessage = (ev) => this.mensaje(ev.data);
    nodo.connect(ctx.destination);
    nodo.port.postMessage({ t: 'wasm', bytes: await this.wasm() });
    this.ctx = ctx;
    this.nodo = nodo;
    return nodo;
  }

  private mensaje(m: { t: string; id?: number; ok?: boolean; texto?: Uint8Array | null } & Respuesta): void {
    if (m.t === 'respuesta' && m.id !== undefined) {
      this.esperas.get(m.id)?.(m);
      this.esperas.delete(m.id);
    } else if (m.t === 'creado') {
      this.alCrear?.({ ok: !!m.ok, texto: m.texto ?? null });
    } else if (m.t === 'error' && m.texto) {
      this.notifyListeners('error', { mensaje: dec.decode(m.texto) });
    } else if (m.t === 'fin') {
      this.publicar({ activo: false, sonando: false, terminado: true, recetaId: null, segundos: 0, duracion: 0 });
      void this.ctx?.suspend();
    }
  }

  private preguntar(orden: Record<string, unknown>): Promise<Respuesta> {
    const nodo = this.nodo;
    if (!nodo) return Promise.resolve({ segundos: 0, duracion: 0, bandas: null });
    const id = ++this.turno;
    return new Promise((listo) => {
      this.esperas.set(id, listo);
      nodo.port.postMessage({ ...orden, id });
    });
  }

  private publicar(cambios: Partial<Estado>): void {
    this.e = { ...this.e, ...cambios };
    this.notifyListeners('estado', this.e);
  }

  async tocar(o: OpcionesTocar): Promise<void> {
    const { motivo } = await this.validar({ receta: o.receta });
    if (motivo) throw new Error(motivo);
    const nodo = await this.audio();
    if (this.suspension) clearTimeout(this.suspension);
    await this.ctx!.resume();
    this.volumenDb = o.volumenDb ?? this.volumenDb;
    this.publicar({ activo: true, sonando: true, cargando: true, terminado: false, recetaId: o.recetaId, segundos: 0, duracion: 0 });

    const lista = await this.conReceta(o.receta, (x, p, n) => {
      if (!x.archivos(p, n)) throw new Error(SonidosWeb.texto(x));
      return SonidosWeb.texto(x).split('\n').filter(Boolean);
    });
    await Promise.all(
      lista
        .filter((a) => !this.cargadas.has(a))
        .map(async (a) => {
          const r = await fetch(`${BASE}audio/${a}`);
          if (!r.ok) throw new Error(`falta audio/${a}`);
          nodo.port.postMessage({ t: 'muestra', nombre: cod.encode(a), ogg: new Uint8Array(await r.arrayBuffer()) });
          this.cargadas.add(a);
        }),
    );
    const creado = new Promise<{ ok: boolean; texto: Uint8Array | null }>((listo) => (this.alCrear = listo));
    nodo.port.postMessage({ t: 'crear', receta: cod.encode(o.receta), duracion: (o.minutos ?? 0) * 60, volumen: this.volumenDb });
    const r = await creado;
    if (!r.ok) {
      this.publicar({ ...EN_REPOSO, auriculares: true });
      throw new Error(r.texto ? dec.decode(r.texto) : 'el núcleo rechazó la receta');
    }
    const c = await this.preguntar({ t: 'consulta' });
    this.publicar({ cargando: false, segundos: c.segundos, duracion: c.duracion ?? 0 });
  }

  async pausar(): Promise<void> {
    if (!this.e.activo || !this.e.sonando) return;
    this.nodo?.port.postMessage({ t: 'atenuar', callar: true });
    const c = await this.preguntar({ t: 'consulta' });
    this.publicar({ sonando: false, segundos: c.segundos });
    // El núcleo calla en 0,4 s; después se suelta el dispositivo de audio.
    this.suspension = setTimeout(() => void this.ctx?.suspend(), 600);
  }

  async reanudar(): Promise<void> {
    if (!this.e.activo || this.e.sonando) return;
    if (this.suspension) clearTimeout(this.suspension);
    await this.ctx?.resume();
    this.nodo?.port.postMessage({ t: 'atenuar', callar: false });
    const c = await this.preguntar({ t: 'consulta' });
    this.publicar({ sonando: true, segundos: c.segundos });
  }

  async parar(): Promise<void> {
    this.nodo?.port.postMessage({ t: 'atenuar', callar: true });
    this.publicar({ activo: false, sonando: false, recetaId: null, segundos: 0, duracion: 0 });
    this.suspension = setTimeout(() => {
      this.nodo?.port.postMessage({ t: 'parar' });
      void this.ctx?.suspend();
    }, 600);
  }

  async extender(o: { minutos: number }): Promise<{ minutos: number }> {
    if (!this.e.activo) return { minutos: 0 };
    const r = await this.preguntar({ t: 'extender', segundos: o.minutos * 60 });
    const c = await this.preguntar({ t: 'consulta' });
    this.publicar({ segundos: c.segundos, duracion: c.duracion ?? this.e.duracion });
    return { minutos: r.segundos / 60 };
  }

  async volumen(o: { db: number }): Promise<void> {
    this.volumenDb = o.db;
    this.nodo?.port.postMessage({ t: 'volumen', db: o.db });
  }

  async mando(o: { hz: number }): Promise<{ hz: number }> {
    return { hz: (await this.preguntar({ t: 'mando', hz: o.hz })).segundos };
  }

  async espectro(o: { bandas: number }): Promise<{ bandas: number[] }> {
    if (!this.e.sonando || this.e.cargando) return { bandas: [] };
    const r = await this.preguntar({ t: 'consulta', bandas: o.bandas });
    return { bandas: r.bandas ? Array.from(r.bandas) : [] };
  }

  async estado(): Promise<Estado> {
    if (this.e.activo && !this.e.cargando) this.e = { ...this.e, segundos: (await this.preguntar({ t: 'consulta' })).segundos };
    return this.e;
  }
}
