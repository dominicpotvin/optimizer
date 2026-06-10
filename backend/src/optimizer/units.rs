// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Conversions d'unites vers/depuis le micrometre (i64), l'unite interne.
//!
//! Le frontend gere la saisie en pi-po / mm ; ces helpers servent aux tests
//! et a une eventuelle conversion cote API.

pub const UM_PER_MM: i64 = 1_000;
pub const UM_PER_INCH: i64 = 25_400; // 1 po = 25.4 mm exact
pub const UM_PER_FOOT: i64 = UM_PER_INCH * 12; // 304_800

/// Millimetres (f64) -> micrometres, arrondi au plus proche.
pub fn mm(v: f64) -> i64 {
    (v * UM_PER_MM as f64).round() as i64
}

/// Pouces (f64) -> micrometres, arrondi au plus proche.
pub fn inch(v: f64) -> i64 {
    (v * UM_PER_INCH as f64).round() as i64
}

/// Pieds (f64) -> micrometres, arrondi au plus proche.
pub fn foot(v: f64) -> i64 {
    (v * UM_PER_FOOT as f64).round() as i64
}

/// Micrometres -> millimetres.
pub fn to_mm(um: i64) -> f64 {
    um as f64 / UM_PER_MM as f64
}

/// Micrometres -> pouces.
pub fn to_inch(um: i64) -> f64 {
    um as f64 / UM_PER_INCH as f64
}

/// Micrometres -> pieds.
pub fn to_foot(um: i64) -> f64 {
    um as f64 / UM_PER_FOOT as f64
}
