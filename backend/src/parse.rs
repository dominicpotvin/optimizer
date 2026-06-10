// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Analyse de longueurs saisies en texte (impérial ou métrique) vers micrometres.
//!
//! Formats acceptes (exemples) :
//!   "5"            -> 5 (unite par defaut)
//!   "5'"           -> 5 pieds
//!   "60\""         -> 60 pouces
//!   "1524mm"       -> 1524 mm
//!   "3-1/2\""      -> 3.5 pouces
//!   "3 1/2 po"     -> 3.5 pouces
//!   "1/8\""        -> 0.125 pouce
//!   "5'6\""        -> 5 pieds 6 pouces
//!   "5' 6-1/2\""   -> 5 pieds 6.5 pouces

use crate::optimizer::units::{foot, inch, mm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Foot,
    Inch,
    Mm,
}

impl Unit {
    pub fn from_str(s: &str) -> Option<Unit> {
        let t = s.trim().to_lowercase();
        match t.as_str() {
            "pi" | "ft" | "foot" | "feet" | "'" | "pied" | "pieds" => Some(Unit::Foot),
            "po" | "in" | "inch" | "inches" | "\"" | "pouce" | "pouces" => Some(Unit::Inch),
            "mm" | "millimetre" | "millimetres" | "millimeter" | "millimeters" => Some(Unit::Mm),
            _ => None,
        }
    }

    pub fn to_um(self, value: f64) -> i64 {
        match self {
            Unit::Foot => foot(value),
            Unit::Inch => inch(value),
            Unit::Mm => mm(value),
        }
    }
}

/// Convertit un nombre simple ("3", "3.5", "1/8", "3-1/2", "3 1/2") en f64.
fn parse_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Nombre mixte: "3-1/2" ou "3 1/2"
    let normalized = s.replace('-', " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    match parts.as_slice() {
        [whole, frac] if frac.contains('/') => {
            let w: f64 = whole.parse().ok()?;
            let f = parse_fraction(frac)?;
            Some(w + f)
        }
        [single] => {
            if single.contains('/') {
                parse_fraction(single)
            } else {
                single.parse().ok()
            }
        }
        _ => None,
    }
}

fn parse_fraction(s: &str) -> Option<f64> {
    let (n, d) = s.split_once('/')?;
    let n: f64 = n.trim().parse().ok()?;
    let d: f64 = d.trim().parse().ok()?;
    if d == 0.0 {
        None
    } else {
        Some(n / d)
    }
}

/// Analyse une longueur en texte -> micrometres. `default_unit` sert quand
/// aucune unite n'est precisee.
pub fn parse_length(raw: &str, default_unit: Unit) -> Option<i64> {
    let s = raw.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    // Cas mm explicite.
    if let Some(idx) = s.find("mm") {
        let num = &s[..idx];
        return parse_number(num).map(|v| Unit::Mm.to_um(v));
    }

    // Cas pieds+pouces combinés: contient ' et/ou "
    let has_feet = s.contains('\'') || s.contains("pi") || s.contains("pied");
    let has_inch = s.contains('"') || s.contains("po") || s.contains("pouce") || s.contains("inch");

    if has_feet || has_inch {
        let mut total_um = 0i64;
        let mut rest = s.clone();

        // Pieds: tout ce qui precede le premier marqueur de pied.
        if let Some(pos) = feet_marker_pos(&rest) {
            let (feet_part, after) = split_at_marker(&rest, pos);
            if let Some(v) = parse_number(&feet_part) {
                total_um += Unit::Foot.to_um(v);
            }
            rest = after;
        }

        // Pouces: le reste (avant un éventuel marqueur de pouce).
        let inch_part = strip_inch_marker(&rest);
        if let Some(v) = parse_number(&inch_part) {
            total_um += Unit::Inch.to_um(v);
        }
        return Some(total_um);
    }

    // Sinon: nombre nu dans l'unite par defaut.
    parse_number(&s).map(|v| default_unit.to_um(v))
}

fn feet_marker_pos(s: &str) -> Option<usize> {
    s.find('\'')
        .or_else(|| s.find("pied"))
        .or_else(|| find_word(s, "pi"))
}

/// Trouve "pi" mais pas dans "pied" deja gere, et evite de matcher dans un mot.
fn find_word(s: &str, w: &str) -> Option<usize> {
    s.find(w)
}

fn split_at_marker(s: &str, pos: usize) -> (String, String) {
    let before = s[..pos].to_string();
    // sauter le marqueur (', "pi", "pied")
    let after = if s[pos..].starts_with('\'') {
        s[pos + 1..].to_string()
    } else if s[pos..].starts_with("pied") {
        s[pos + 4..].to_string()
    } else if s[pos..].starts_with("pi") {
        s[pos + 2..].to_string()
    } else {
        s[pos..].to_string()
    };
    (before, after)
}

fn strip_inch_marker(s: &str) -> String {
    s.replace('"', "")
        .replace("pouces", "")
        .replace("pouce", "")
        .replace("inches", "")
        .replace("inch", "")
        .replace("po", "")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::units::{foot, inch, mm};

    #[test]
    fn nombre_nu_utilise_unite_defaut() {
        assert_eq!(parse_length("5", Unit::Foot), Some(foot(5.0)));
        assert_eq!(parse_length("60", Unit::Inch), Some(inch(60.0)));
        assert_eq!(parse_length("1524", Unit::Mm), Some(mm(1524.0)));
    }

    #[test]
    fn unites_explicites() {
        assert_eq!(parse_length("5'", Unit::Inch), Some(foot(5.0)));
        assert_eq!(parse_length("60\"", Unit::Foot), Some(inch(60.0)));
        assert_eq!(parse_length("1524mm", Unit::Foot), Some(mm(1524.0)));
        assert_eq!(parse_length("3 pi", Unit::Mm), Some(foot(3.0)));
    }

    #[test]
    fn fractions_et_mixtes() {
        assert_eq!(parse_length("1/8\"", Unit::Inch), Some(inch(0.125)));
        assert_eq!(parse_length("3-1/2\"", Unit::Inch), Some(inch(3.5)));
        assert_eq!(parse_length("3 1/2 po", Unit::Foot), Some(inch(3.5)));
    }

    #[test]
    fn pieds_et_pouces_combines() {
        assert_eq!(parse_length("5'6\"", Unit::Inch), Some(foot(5.0) + inch(6.0)));
        assert_eq!(
            parse_length("5' 6-1/2\"", Unit::Inch),
            Some(foot(5.0) + inch(6.5))
        );
    }
}
