// El núcleo (sonidos.wasm) dentro de un AudioWorklet: el hilo de audio del navegador.
// Aquí no hay TextEncoder ni fetch: el hilo principal manda todo ya en bytes y recibe los
// textos (errores) en bytes también.
class Sonidos extends AudioWorkletProcessor {
  constructor() {
    super();
    this.x = null;
    this.suena = false;
    this.avisado = false;
    this.port.onmessage = (e) => this.orden(e.data);
  }

  meter(bytes) {
    const p = this.x.reservar(bytes.length);
    new Uint8Array(this.x.memory.buffer, p, bytes.length).set(bytes);
    return p;
  }

  texto() {
    return new Uint8Array(this.x.memory.buffer, this.x.texto_ptr(), this.x.texto_len()).slice();
  }

  orden(m) {
    const x = this.x;
    switch (m.t) {
      case 'wasm':
        this.x = new WebAssembly.Instance(new WebAssembly.Module(m.bytes), {}).exports;
        break;
      case 'muestra': {
        const n = this.meter(m.nombre);
        const o = this.meter(m.ogg);
        const ok = x.banco_agregar(n, m.nombre.length, o, m.ogg.length);
        x.liberar(n, m.nombre.length);
        x.liberar(o, m.ogg.length);
        if (!ok) this.port.postMessage({ t: 'error', texto: this.texto() });
        break;
      }
      case 'crear': {
        const p = this.meter(m.receta);
        const ok = x.crear(p, m.receta.length, m.duracion);
        x.liberar(p, m.receta.length);
        if (ok) {
          x.volumen(m.volumen);
          this.suena = true;
          this.avisado = false;
        }
        this.port.postMessage({ t: 'creado', ok: !!ok, texto: ok ? null : this.texto() });
        break;
      }
      case 'atenuar':
        x.atenuar(m.callar ? 1 : 0);
        break;
      case 'volumen':
        x.volumen(m.db);
        break;
      case 'extender':
        this.port.postMessage({ t: 'respuesta', id: m.id, segundos: x.extender(m.segundos) });
        break;
      case 'parar':
        this.suena = false;
        x.parar();
        break;
      case 'consulta': {
        let bandas = null;
        if (m.bandas && this.suena) bandas = new Float32Array(x.memory.buffer, x.espectro(m.bandas), m.bandas).slice();
        this.port.postMessage({ t: 'respuesta', id: m.id, segundos: x.segundos(), duracion: x.duracion(), bandas });
        break;
      }
    }
  }

  process(_entradas, salidas) {
    const [l, r] = salidas[0];
    if (!this.x || !this.suena || !l) return true;
    const n = l.length;
    const p = this.x.procesar(n);
    if (!p) return true;
    const v = new Float32Array(this.x.memory.buffer, p, n * 2);
    for (let i = 0; i < n; i++) {
      l[i] = v[i * 2];
      if (r) r[i] = v[i * 2 + 1];
    }
    if (this.x.estado() & 1 && !this.avisado) {
      this.avisado = true;
      this.suena = false;
      this.port.postMessage({ t: 'fin' });
    }
    return true;
  }
}

registerProcessor('sonidos', Sonidos);
