//! Nivel, ventana y desvanecimientos de cada capa, y el máster con techo.

use crate::dsp::{suave, GananciaSuave, Limitador};
use crate::receta::{Capa, Receta};

/// Estado de mezcla de una capa: su objetivo de ganancia (nivel × ventana × fades) pasa
/// por `GananciaSuave`, que impide subidas bruscas.
pub struct CapaMezcla {
    nivel_db: f32,
    desde_s: f32,
    hasta_s: f32,
    /// La capa dura hasta el final de la receta (y se alarga si la receta se alarga).
    abierta: bool,
    transicion_s: f32,
    suave: GananciaSuave,
}

impl CapaMezcla {
    pub fn nueva(capa: &Capa, receta: &Receta, sr: f32) -> Self {
        CapaMezcla {
            nivel_db: capa.nivel_db,
            desde_s: capa.desde_s,
            hasta_s: capa.hasta_s.unwrap_or(receta.duracion_s),
            abierta: capa.hasta_s.is_none(),
            transicion_s: capa.transicion_s,
            suave: GananciaSuave::nueva(sr),
        }
    }

    pub fn extender(&mut self, segundos: f32) {
        if self.abierta {
            self.hasta_s += segundos;
        }
    }

    /// Ganancia objetivo en dB en el instante `t` (antes del suavizado).
    pub fn objetivo_db(&self, t: f32) -> f32 {
        if t < self.desde_s || t > self.hasta_s {
            return GananciaSuave::SILENCIO_DB;
        }
        let entrada = suave((t - self.desde_s) / self.transicion_s);
        // Una capa abierta no se apaga sola: de su final se encarga el máster.
        let salida = if self.abierta { 1.0 } else { suave((self.hasta_s - t) / self.transicion_s) };
        let v = entrada.min(salida);
        if v >= 1.0 {
            self.nivel_db
        } else if v <= 0.0 {
            GananciaSuave::SILENCIO_DB
        } else {
            self.nivel_db + 20.0 * v.log10()
        }
    }

    #[inline]
    pub fn ganancia(&mut self, t: f32) -> f32 {
        let obj = self.objetivo_db(t);
        self.suave.hacia(obj)
    }
}

/// Máster: desvanecimiento global y limitador con techo.
pub struct Master {
    fade_in_s: f32,
    fade_out_s: f32,
    duracion_s: f32,
    limitador: Limitador,
}

impl Master {
    pub fn nuevo(receta: &Receta, sr: f32) -> Self {
        Master {
            fade_in_s: receta.fade_in_s,
            fade_out_s: receta.fade_out_s,
            duracion_s: receta.duracion_s,
            limitador: Limitador::nuevo(receta.techo_db, sr),
        }
    }

    pub fn extender(&mut self, segundos: f32) {
        self.duracion_s += segundos;
    }

    #[inline]
    pub fn envolvente(&self, t: f32) -> f32 {
        let a = suave(t / self.fade_in_s);
        let b = suave((self.duracion_s - t) / self.fade_out_s);
        a.min(b)
    }

    #[inline]
    pub fn procesar(&mut self, t: f32, l: f32, r: f32) -> (f32, f32) {
        let e = self.envolvente(t);
        self.limitador.procesar(l * e, r * e)
    }
}
