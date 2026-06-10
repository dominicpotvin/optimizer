// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Moteur d'optimisation de coupe (1D cutting stock).

pub mod model;
pub mod solver;
pub mod units;

pub use model::*;
pub use solver::solve;

#[cfg(test)]
mod tests {
    use super::units::*;
    use super::*;

    fn stock(id: &str, label: &str, length_um: i64, available: Option<u32>) -> StockType {
        StockType {
            id: id.into(),
            label: label.into(),
            length_um,
            available,
        }
    }

    fn part(id: &str, label: &str, length_um: i64, qty: u32) -> PartType {
        PartType {
            id: id.into(),
            label: label.into(),
            length_um,
            qty,
            model: None,
        }
    }

    /// Verifie l'integrite physique de chaque barre d'une solution.
    fn assert_feasible(sol: &Solution, kerf: i64) {
        for b in &sol.bars {
            let sum_cuts: i64 = b.cuts.iter().map(|c| c.length_um).sum();
            let kerf_total = kerf * b.cuts.len() as i64;
            assert_eq!(sum_cuts, b.used_um, "used_um incoherent");
            assert_eq!(kerf_total, b.kerf_total_um, "kerf_total incoherent");
            assert_eq!(
                b.offcut_um,
                b.stock_length_um - sum_cuts - kerf_total,
                "offcut incoherent"
            );
            assert!(b.offcut_um >= 0, "chute negative -> barre sur-remplie !");
            assert!(
                sum_cuts + kerf_total <= b.stock_length_um,
                "barre sur-remplie"
            );
        }
    }

    /// Verifie que toutes les pieces demandees sont produites, ni plus ni moins.
    fn assert_demand_met(sol: &Solution, parts: &[PartType]) {
        use std::collections::HashMap;
        let mut want: HashMap<&str, u32> = HashMap::new();
        for p in parts {
            *want.entry(p.id.as_str()).or_insert(0) += p.qty;
        }
        let mut got: HashMap<&str, u32> = HashMap::new();
        for b in &sol.bars {
            for c in &b.cuts {
                *got.entry(c.part_id.as_str()).or_insert(0) += 1;
            }
        }
        for u in &sol.unplaced {
            *got.entry(u.part_id.as_str()).or_insert(0) += 1;
        }
        assert_eq!(want, got, "le compte de pieces ne correspond pas a la demande");
    }

    #[test]
    fn exemple_de_reference_16pi() {
        // 16 pi, kerf 1/8", liste: 1x3pi, 1x4pi, 2x6pi, 6x5pi
        let problem = Problem {
            stocks: vec![stock("s16", "16 pi", foot(16.0), None)],
            parts: vec![
                part("p3", "3 pi", foot(3.0), 1),
                part("p4", "4 pi", foot(4.0), 1),
                part("p6", "6 pi", foot(6.0), 2),
                part("p5", "5 pi", foot(5.0), 6),
            ],
            settings: Settings {
                kerf_um: inch(0.125),
                reusable_threshold_um: foot(1.0), // chute >= 1 pi = reutilisable
                time_limit_ms: Some(500),
            },
        };
        let sol = solve(&problem);
        assert!(sol.complete, "toutes les pieces doivent etre placees");
        assert_feasible(&sol, problem.settings.kerf_um);
        assert_demand_met(&sol, &problem.parts);

        // Total des pieces = 36+48+72*2+60*6 = 588 po. Borne basse barres:
        // 588 po / 192 po = 3.06 -> au moins 4 barres de 16 pi.
        let lower_bound = 4;
        assert!(
            sol.summary.total_bars >= lower_bound,
            "moins que la borne basse, impossible"
        );
        // Une bonne solution tient en 4 barres.
        assert_eq!(
            sol.summary.total_bars, 4,
            "attendu 4 barres de 16 pi, obtenu {}",
            sol.summary.total_bars
        );

        println!("--- Exemple de reference (16 pi) ---");
        for (i, b) in sol.bars.iter().enumerate() {
            let pieces: Vec<String> =
                b.cuts.iter().map(|c| c.label.clone()).collect();
            println!(
                "Barre {} [{}]: {} | chute {:.3} po{}",
                i + 1,
                b.stock_label,
                pieces.join(" + "),
                to_inch(b.offcut_um),
                if b.reusable { " (reutilisable)" } else { "" }
            );
        }
        println!(
            "Total: {} barres, utilisation {:.1}%, perte reelle {:.2} po, chute reutilisable {:.2} po",
            sol.summary.total_bars,
            sol.summary.utilization_pct,
            to_inch(sol.summary.real_waste_um),
            to_inch(sol.summary.reusable_offcut_um),
        );
    }

    #[test]
    fn multi_longueurs_choisit_le_meilleur_mix() {
        // Pieces de 5 pi ; barres de 10 pi vs 16 pi disponibles.
        // 2x5pi tient pile dans 10 pi (avec kerf) -> devrait preferer.
        let problem = Problem {
            stocks: vec![
                stock("s10", "10 pi", foot(10.0), None),
                stock("s16", "16 pi", foot(16.0), None),
            ],
            parts: vec![part("p5", "5 pi", foot(5.0), 4)],
            settings: Settings {
                kerf_um: inch(0.125),
                reusable_threshold_um: foot(2.0),
                time_limit_ms: Some(500),
            },
        };
        let sol = solve(&problem);
        assert!(sol.complete);
        assert_feasible(&sol, problem.settings.kerf_um);
        assert_demand_met(&sol, &problem.parts);
        // 2x5pi (120 po) ne tiennent PAS dans 10 pi a cause du kerf (+0.25 po).
        // Optimum reel : 1x16 pi (3 pieces) + 1x10 pi (1 piece) = 312 po.
        assert_eq!(sol.summary.total_bars, 2);
        assert_eq!(sol.summary.total_stock_um, foot(26.0));
        let mut labels: Vec<&str> = sol.bars.iter().map(|b| b.stock_label.as_str()).collect();
        labels.sort();
        assert_eq!(labels, vec!["10 pi", "16 pi"]);
    }

    #[test]
    fn dispo_limitee_laisse_des_pieces_non_placees() {
        let problem = Problem {
            stocks: vec![stock("s16", "16 pi", foot(16.0), Some(1))],
            parts: vec![part("p10", "10 pi", foot(10.0), 2)],
            settings: Settings {
                kerf_um: inch(0.125),
                reusable_threshold_um: 0,
                time_limit_ms: Some(200),
            },
        };
        let sol = solve(&problem);
        // Une seule barre dispo, 2 pieces de 10 pi n'y tiennent pas ensemble.
        assert!(!sol.complete);
        assert_eq!(sol.unplaced.len(), 1);
        assert_eq!(sol.summary.total_bars, 1);
        assert_feasible(&sol, problem.settings.kerf_um);
        assert_demand_met(&sol, &problem.parts);
    }

    #[test]
    fn piece_trop_longue_est_non_placee() {
        let problem = Problem {
            stocks: vec![stock("s16", "16 pi", foot(16.0), None)],
            parts: vec![part("pbig", "20 pi", foot(20.0), 1)],
            settings: Settings::default(),
        };
        let sol = solve(&problem);
        assert!(!sol.complete);
        assert_eq!(sol.unplaced.len(), 1);
        assert_eq!(sol.summary.total_bars, 0);
    }

    #[test]
    fn kerf_respecte_la_capacite() {
        // 2 pieces de 8 pi dans une barre de 16 pi : avec kerf, ca NE rentre PAS
        // (8+8 = 16 mais 2 kerfs en plus depassent).
        let problem = Problem {
            stocks: vec![stock("s16", "16 pi", foot(16.0), None)],
            parts: vec![part("p8", "8 pi", foot(8.0), 2)],
            settings: Settings {
                kerf_um: inch(0.125),
                reusable_threshold_um: 0,
                time_limit_ms: Some(200),
            },
        };
        let sol = solve(&problem);
        assert!(sol.complete);
        assert_feasible(&sol, problem.settings.kerf_um);
        // Doit utiliser 2 barres car le kerf empeche 8+8 sur une seule.
        assert_eq!(sol.summary.total_bars, 2);
    }

    #[test]
    fn metrique_fonctionne_aussi() {
        // Barre 6000 mm, kerf 3 mm, pieces 1500 mm x 4 -> 1 barre (4*1500=6000,
        // mais kerf 4*3=12 -> 6012 > 6000 -> 3 par barre).
        let problem = Problem {
            stocks: vec![stock("s6000", "6000 mm", mm(6000.0), None)],
            parts: vec![part("p1500", "1500 mm", mm(1500.0), 4)],
            settings: Settings {
                kerf_um: mm(3.0),
                reusable_threshold_um: mm(500.0),
                time_limit_ms: Some(200),
            },
        };
        let sol = solve(&problem);
        assert!(sol.complete);
        assert_feasible(&sol, problem.settings.kerf_um);
        assert_demand_met(&sol, &problem.parts);
        assert_eq!(sol.summary.total_bars, 2);
    }
}
