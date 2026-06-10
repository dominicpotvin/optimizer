//! Generation des fichiers CSV : modele d'import et export du plan de coupe.

use crate::api::PartDrilling;
use crate::optimizer::units::{to_foot, to_inch, to_mm};
use crate::optimizer::{PartType, Solution};
use crate::parse::Unit;

/// Modele a remplir dans Excel puis re-importer.
pub fn template_csv() -> String {
    let mut s = String::new();
    s.push_str("jobID,partID,model,longueur,quantite,unite\n");
    s.push_str("JOB-001,P1,Profil-U-100,6,2,pi\n");
    s.push_str("JOB-001,P2,Profil-U-100,5,6,pi\n");
    s.push_str("JOB-001,P3,Profil-U-100,4,1,pi\n");
    s.push_str("JOB-001,P4,Profil-U-100,3,1,pi\n");
    s
}

fn unit_label(u: Unit) -> &'static str {
    match u {
        Unit::Foot => "pi",
        Unit::Inch => "po",
        Unit::Mm => "mm",
    }
}

fn conv(um: i64, u: Unit) -> String {
    match u {
        Unit::Foot => format!("{:.3}", to_foot(um)),
        Unit::Inch => format!("{:.3}", to_inch(um)),
        Unit::Mm => format!("{:.1}", to_mm(um)),
    }
}

/// Export plat du plan de coupe : une ligne par piece coupee. Les colonnes au
/// niveau de la barre (chute, reutilisable) sont repetees pour faciliter le
/// filtrage/tableau croise dans Excel.
pub fn solution_to_csv(job_number: &str, sol: &Solution, parts: &[PartType], unit: Unit) -> String {
    let model_of = |part_id: &str| -> String {
        parts
            .iter()
            .find(|p| p.id == part_id)
            .and_then(|p| p.model.clone())
            .unwrap_or_default()
    };

    let mut wtr = csv::Writer::from_writer(vec![]);
    let u = unit_label(unit);
    wtr.write_record([
        "numero_job",
        "barre",
        "type_stock",
        &format!("longueur_stock_{u}"),
        "piece_dans_barre",
        "partID",
        "model",
        &format!("longueur_piece_{u}"),
        &format!("chute_barre_{u}"),
        "reutilisable",
    ])
    .ok();

    for (bi, bar) in sol.bars.iter().enumerate() {
        for (ci, cut) in bar.cuts.iter().enumerate() {
            wtr.write_record([
                job_number,
                &(bi + 1).to_string(),
                &bar.stock_label,
                &conv(bar.stock_length_um, unit),
                &(ci + 1).to_string(),
                &cut.part_id,
                &model_of(&cut.part_id),
                &conv(cut.length_um, unit),
                &conv(bar.offcut_um, unit),
                if bar.reusable { "oui" } else { "non" },
            ])
            .ok();
        }
    }

    let bytes = wtr.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

/// Export du plan de percage : une ligne par trou, positions depuis chaque bout.
pub fn drilling_to_csv(results: &[PartDrilling], unit: Unit) -> String {
    let mut wtr = csv::Writer::from_writer(vec![]);
    let u = unit_label(unit);
    wtr.write_record([
        "partID",
        "model",
        &format!("longueur_{u}"),
        "nb_trous",
        &format!("marge_bout_{u}"),
        "trou",
        &format!("depuis_A_{u}"),
        &format!("depuis_B_{u}"),
        "centre",
    ])
    .ok();

    for d in results {
        let model = d.model.clone().unwrap_or_default();
        if !d.result.ok {
            wtr.write_record([
                &d.part_id,
                &model,
                &conv(d.length_um, unit),
                "0",
                "",
                "IMPOSSIBLE",
                "",
                "",
                "",
            ])
            .ok();
            continue;
        }
        for h in &d.result.holes {
            wtr.write_record([
                &d.part_id,
                &model,
                &conv(d.length_um, unit),
                &d.result.n.to_string(),
                &conv(d.result.marge_um, unit),
                &h.index.to_string(),
                &conv(h.from_a_um, unit),
                &conv(h.from_b_um, unit),
                &String::from(if h.is_center { "oui" } else { "non" }),
            ])
            .ok();
        }
    }

    let bytes = wtr.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}
