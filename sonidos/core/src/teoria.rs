//! Lo poco de teoría musical que necesita el colchón: modos como ciclos de acordes.
//! Un acorde es una lista de semitonos respecto a la raíz (negativos = por debajo).

pub fn ciclo_de_acordes(modo: &str) -> Vec<Vec<i32>> {
    match modo {
        // Re pentatónico mayor (D E F# A B), el agua de Lienzo. Sin tensiones.
        "re_pentatonico" | "pentatonico" => vec![
            vec![0, 7, 12, 16, 19],     // I   D A D F# A
            vec![-3, 4, 9, 12, 16],     // vi  B F# B D F#
            vec![2, 9, 14, 19, 21],     // ii  E B E A B
            vec![-5, 2, 7, 12, 14],     // V   A E A D E
        ],
        // Menor suave: i, VI, III, VII. Más hondo, para dormir.
        "menor" => vec![
            vec![0, 7, 12, 15, 19],     // i
            vec![-4, 3, 8, 12, 15],     // VI
            vec![-9, -2, 3, 7, 10],     // III
            vec![-2, 5, 10, 14, 17],    // VII
        ],
        // Quintas: un bordón con su quinta y octavas. Lo más neutro; para afinaciones
        // "especiales" (432, solfeggio) donde la raíz es lo que importa.
        _ => vec![vec![0, 7, 12, 19, 24], vec![0, 7, 12, 17, 24]],
    }
}

/// Frecuencia de la raíz por defecto (Re2) para un La4 dado.
pub fn raiz_por_defecto(afinacion_hz: f32) -> f32 {
    afinacion_hz * 2f32.powf(-31.0 / 12.0)
}

pub fn semitonos(raiz_hz: f32, st: i32) -> f32 {
    raiz_hz * 2f32.powf(st as f32 / 12.0)
}
