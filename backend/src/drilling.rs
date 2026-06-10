//! Repartition symetrique des trous de percage sur une piece, selon le
//! cahier des charges d'origine (calcul-percage). Port fidele de l'algorithme,
//! en arithmetique micrometres (i64).
//!
//! Regles :
//! - symetrie autour du centre (centre = longueur / 2) ;
//! - entraxe centre-a-centre `pas` constant ;
//! - marge de bout (bord -> centre du trou extreme) dans [marge_min, marge_max] ;
//! - N ajuste automatiquement ; N impair -> trou central, N pair -> encadre le centre ;
//! - on retient le N donnant la plus grande marge valide (centrage maximal).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DrillingParams {
    /// Entraxe centre-a-centre, en micrometres.
    pub pas_um: i64,
    /// Marge de bout minimale (bord -> 1er trou), en micrometres.
    pub marge_min_um: i64,
    /// Marge de bout maximale, en micrometres.
    pub marge_max_um: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Hole {
    pub index: usize,
    /// Centre du trou mesure depuis l'extremite A, en micrometres.
    pub from_a_um: i64,
    /// Centre du trou mesure depuis l'extremite B, en micrometres.
    pub from_b_um: i64,
    pub is_center: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct DrillResult {
    pub ok: bool,
    pub message: Option<String>,
    /// Nombre de trous.
    pub n: usize,
    /// Marge de bout effective (point de depart depuis chaque extremite), en um.
    pub marge_um: i64,
    pub pas_um: i64,
    /// "impair (trou central)" | "pair (encadre le centre)"
    pub parity: String,
    pub holes: Vec<Hole>,
}

impl DrillResult {
    fn impossible(msg: impl Into<String>) -> Self {
        DrillResult {
            ok: false,
            message: Some(msg.into()),
            n: 0,
            marge_um: 0,
            pas_um: 0,
            parity: String::new(),
            holes: Vec::new(),
        }
    }
}

/// Calcule la repartition pour une piece de longueur `length_um`.
pub fn compute(length_um: i64, p: &DrillingParams) -> DrillResult {
    let l = length_um;
    let pas = p.pas_um;
    let m_min = p.marge_min_um;
    let m_max = p.marge_max_um;

    if l <= 0 || pas <= 0 || m_min < 0 || m_max < m_min {
        return DrillResult::impossible("Parametres de percage invalides.");
    }

    // N theorique max : quand marge = marge_min -> span = L - 2*marge_min.
    let span_max = l - 2 * m_min;
    let mut nmax = if span_max < 0 { 0 } else { span_max / pas + 1 };
    if nmax < 1 {
        nmax = 1;
    }

    // Candidats : marge dans [m_min, m_max]  <=>  rem = L - span dans [2*m_min, 2*m_max].
    // (rem = 2 * marge, garde en entier pour l'exactitude.)
    let mut best: Option<(i64, i64)> = None; // (N, rem) avec rem le plus grand
    for n in 1..=(nmax + 1) {
        let span = (n - 1) * pas;
        let rem = l - span;
        if rem >= 2 * m_min && rem <= 2 * m_max {
            match best {
                Some((_, best_rem)) if best_rem >= rem => {}
                _ => best = Some((n, rem)),
            }
        }
    }

    let (n, rem) = match best {
        Some(v) => v,
        None => {
            return DrillResult::impossible(format!(
                "Aucune repartition possible : avec un entraxe et des marges donnees, \
                 la longueur ne permet pas de placer les trous extremes dans la plage autorisee."
            ))
        }
    };

    let marge_um = rem / 2;
    let centre = l as f64 / 2.0;
    let pasf = pas as f64;

    // Positions (en um, depuis A), calculees en f64 puis arrondies.
    let mut positions: Vec<f64> = Vec::new();
    if n % 2 == 1 {
        let k = (n - 1) / 2;
        for i in -k..=k {
            positions.push(centre + (i as f64) * pasf);
        }
    } else {
        let demi = n / 2;
        for j in -demi..=demi {
            if j == 0 {
                continue;
            }
            let signe = if j < 0 { -1.0 } else { 1.0 };
            let rang = (j.abs() as f64) - 0.5;
            positions.push(centre + signe * rang * pasf);
        }
    }
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let holes: Vec<Hole> = positions
        .iter()
        .enumerate()
        .map(|(i, &pos_a)| {
            let from_a = pos_a.round() as i64;
            Hole {
                index: i + 1,
                from_a_um: from_a,
                from_b_um: l - from_a, // garantit la symetrie exacte
                is_center: (pos_a - centre).abs() < 0.5,
            }
        })
        .collect();

    let parity = if n % 2 == 1 {
        "impair (trou central)".to_string()
    } else {
        "pair (encadre le centre)".to_string()
    };

    DrillResult {
        ok: true,
        message: None,
        n: n as usize,
        marge_um,
        pas_um: pas,
        parity,
        holes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::units::mm;

    fn params(pas: f64, mn: f64, mx: f64) -> DrillingParams {
        DrillingParams {
            pas_um: mm(pas),
            marge_min_um: mm(mn),
            marge_max_um: mm(mx),
        }
    }

    #[test]
    fn exemple_reference_2400mm() {
        // Cahier des charges : 2400 mm, entraxe 112, marges [40, 112].
        let r = compute(mm(2400.0), &params(112.0, 40.0, 112.0));
        assert!(r.ok);
        assert_eq!(r.n, 21);
        assert_eq!(r.marge_um, mm(80.0)); // marge de bout = 80 mm
        assert_eq!(r.parity, "impair (trou central)");
        // Trou central exactement a 1200 mm des deux bouts.
        let centre = r.holes.iter().find(|h| h.is_center).unwrap();
        assert_eq!(centre.from_a_um, mm(1200.0));
        assert_eq!(centre.from_b_um, mm(1200.0));
        // Symetrie : 1er depuis A == dernier depuis B.
        assert_eq!(r.holes[0].from_a_um, r.holes[r.holes.len() - 1].from_b_um);
        // Premier trou = marge de bout.
        assert_eq!(r.holes[0].from_a_um, mm(80.0));
    }

    #[test]
    fn symetrie_invariante() {
        let r = compute(mm(2000.0), &params(112.0, 40.0, 112.0));
        assert!(r.ok);
        for h in &r.holes {
            assert_eq!(h.from_a_um + h.from_b_um, mm(2000.0));
        }
        // miroir
        let n = r.holes.len();
        for i in 0..n {
            assert_eq!(r.holes[i].from_a_um, r.holes[n - 1 - i].from_b_um);
        }
    }

    #[test]
    fn cas_impossible() {
        // Longueur tres courte : aucune marge valide possible.
        let r = compute(mm(150.0), &params(112.0, 40.0, 50.0));
        // span=0 -> marge=75 > 50 ; span=112 -> marge=19 < 40 -> impossible
        assert!(!r.ok);
        assert!(r.holes.is_empty());
    }
}
