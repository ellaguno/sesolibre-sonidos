//! Nivel, ventana y desvanecimientos de cada capa, y el máster con techo.

use crate::dsp::{suave, GananciaSuave, Limitador};
use crate::receta::{Capa, Receta};

/// Estado de mezcla de una capa: su objetivo de ganancia (nivel × ventana × fades) pasa
/// por `GananciaSuave`, que impide subidas bruscas.
pub struct CapaMezcla {
    nivel_db: f32,
    desde_s: f32,
    hasta_s: f32,
    transicion_s: f32,
    suave: GananciaSuave,
}

impl CapaMezcla {
    pub fn nueva(capa: &Capa, receta: &Receta, sr: f32) -> Self {
        CapaMezcla {
            nivel_db: capa.nivel_db,
            desde_s: capa.desde_s,
            hasta_s: capa.hasta_s.unwrap_or(receta.duracion_s),
            transicion_s: capa.transicion_s,
            suave: GananciaSuave::nueva(sr),
        }
    }

    /// Ganancia lineal objetivo en el instante `t` (antes del suavizado).
    pub fn objetivo_db(&self, t: f32) -> f32 {
        if t < self.desde_s || t > self.hasta_s {
            return GananciaSuave::SILENCIO_DB;
        }
        let entrada = suave((t - self.desde_s) / self.transicion_s);
        let salida = suave((self.hasta_s - t) / self.transicion_s);
        let v = entrada.min(salida);
        if v <= 0.0 {
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
