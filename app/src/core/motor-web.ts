import { WebPlugin } from '@capacitor/core';

import { EN_REPOSO, type Estado, type OpcionesTocar, type SonidosPlugin } from './motor';

/** Simulación para el navegador: mismo contrato, reloj real, sin sonido. */
export class SonidosWeb extends WebPlugin implements Omit<SonidosPlugin, 'addListener'> {
  private e: Estado = { ...EN_REPOSO, simulado: true };
  private inicio = 0;
  private acumulado = 0;
  private fin: ReturnType<typeof setTimeout> | null = null;

  private ahora(): number {
    return this.acumulado + (this.e.sonando ? (Date.now() - this.inicio) / 1000 : 0);
  }

  private publicar(cambios: Partial<Estado>): void {
    this.e = { ...this.e, ...cambios, segundos: cambios.segundos ?? this.ahora() };
    this.notifyListeners('estado', this.e);
    if (this.fin) clearTimeout(this.fin);
    this.fin = null;
    if (this.e.sonando) {
      this.fin = setTimeout(
        () => {
          this.acumulado = 0;
          this.publicar({ activo: false, sonando: false, terminado: true, recetaId: null, segundos: 0, duracion: 0 });
        },
        Math.max(0, this.e.duracion - this.e.segundos) * 1000,
      );
    }
  }

  async tocar(o: OpcionesTocar): Promise<void> {
    const receta = JSON.parse(o.receta) as { duracion_s: number };
    this.acumulado = 0;
    this.inicio = Date.now();
    this.publicar({
      activo: true,
      sonando: true,
      terminado: false,
      recetaId: o.recetaId,
      segundos: 0,
      duracion: o.minutos ? o.minutos * 60 : receta.duracion_s,
    });
  }

  async pausar(): Promise<void> {
    if (!this.e.activo || !this.e.sonando) return;
    this.acumulado = this.ahora();
    this.e.sonando = false;
    this.publicar({ sonando: false, segundos: this.acumulado });
  }

  async reanudar(): Promise<void> {
    if (!this.e.activo || this.e.sonando) return;
    this.inicio = Date.now();
    this.publicar({ sonando: true, segundos: this.acumulado });
  }

  async parar(): Promise<void> {
    this.acumulado = 0;
    this.e.sonando = false;
    this.publicar({ activo: false, sonando: false, recetaId: null, segundos: 0, duracion: 0 });
  }

  async extender(o: { minutos: number }): Promise<{ minutos: number }> {
    if (!this.e.activo) return { minutos: 0 };
    this.publicar({ duracion: this.e.duracion + o.minutos * 60 });
    return { minutos: o.minutos };
  }

  async volumen(): Promise<void> {}

  /** Sin sonido no hay espectro real: una forma inventada que se mueve, para ver el gráfico. */
  async espectro(o: { bandas: number }): Promise<{ bandas: number[] }> {
    if (!this.e.sonando) return { bandas: [] };
    const t = Date.now() / 1000;
    const campana = (x: number, centro: number, ancho: number) => Math.exp(-(((x - centro) / ancho) ** 2));
    return {
      bandas: Array.from({ length: o.bandas }, (_, i) => {
        const x = i / o.bandas;
        const v =
          0.75 * campana(x, 0.22, 0.09) * (0.8 + 0.2 * Math.sin(t * 0.7)) +
          0.55 * campana(x, 0.43, 0.05) * (0.6 + 0.4 * Math.sin(t * 1.3 + 1)) +
          0.3 * campana(x, 0.66, 0.12) * (0.5 + 0.5 * Math.sin(t * 2.1 + 2)) +
          0.04 * Math.random();
        return Math.min(1, v);
      }),
    };
  }

  /** En el navegador no está el núcleo: solo se comprueba que sea una receta con capas. */
  async validar(o: { receta: string }): Promise<{ motivo: string | null }> {
    try {
      const r = JSON.parse(o.receta) as { version?: number; capas?: unknown[] };
      if (r.version !== 1 || !Array.isArray(r.capas) || r.capas.length === 0) return { motivo: 'no es una receta de Sonidos' };
      return { motivo: null };
    } catch {
      return { motivo: 'no es una receta de Sonidos' };
    }
  }

  async estado(): Promise<Estado> {
    return { ...this.e, segundos: this.ahora() };
  }
}
