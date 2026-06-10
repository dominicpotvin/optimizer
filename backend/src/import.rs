// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Import d'une liste de coupe depuis un fichier Excel (.xlsx) ou CSV.
//!
//! Colonnes reconnues (entetes insensibles a la casse/accents/espaces) :
//!   jobID, partID, model, longueur, quantite (optionnel), unite (optionnel).
//! Si `quantite` est absente, chaque ligne compte pour 1 piece.

use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};

use crate::parse::{parse_length, Unit};

pub enum FileKind {
    Xlsx,
    Csv,
}

pub fn detect_kind(filename: &str) -> FileKind {
    if filename.to_lowercase().ends_with(".csv") {
        FileKind::Csv
    } else {
        FileKind::Xlsx
    }
}

#[derive(Debug, Clone)]
pub struct ImportRow {
    pub job_id: Option<String>,
    pub part_id: String,
    pub model: Option<String>,
    pub length_raw: String,
    pub length_um: i64,
    pub qty: u32,
}

pub fn parse_file(
    bytes: &[u8],
    kind: FileKind,
    default_unit: Unit,
) -> Result<Vec<ImportRow>, String> {
    let grid = match kind {
        FileKind::Xlsx => read_xlsx(bytes)?,
        FileKind::Csv => read_csv(bytes)?,
    };
    rows_to_items(grid, default_unit)
}

fn read_xlsx(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut wb: Xlsx<_> =
        open_workbook_from_rs(cursor).map_err(|e| format!("fichier Excel illisible : {e}"))?;
    let range = wb
        .worksheet_range_at(0)
        .ok_or_else(|| "aucune feuille dans le classeur".to_string())?
        .map_err(|e| format!("feuille illisible : {e}"))?;
    let mut out = Vec::new();
    for row in range.rows() {
        out.push(row.iter().map(cell_to_string).collect());
    }
    Ok(out)
}

fn cell_to_string(c: &Data) -> String {
    match c {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => fmt_num(*f),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        other => format!("{other}"),
    }
}

fn fmt_num(f: f64) -> String {
    if f.fract() == 0.0 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}

fn read_csv(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let delim = sniff_delimiter(bytes);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delim)
        .from_reader(bytes);
    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| format!("CSV illisible : {e}"))?;
        out.push(rec.iter().map(|s| s.to_string()).collect());
    }
    Ok(out)
}

/// Devine le separateur CSV (virgule, point-virgule ou tabulation) en comptant
/// les occurrences sur la premiere ligne.
fn sniff_delimiter(bytes: &[u8]) -> u8 {
    let first: &[u8] = match bytes.iter().position(|&b| b == b'\n') {
        Some(i) => &bytes[..i],
        None => bytes,
    };
    let count = |d: u8| first.iter().filter(|&&b| b == d).count();
    let (c, sc, tab) = (count(b','), count(b';'), count(b'\t'));
    if sc >= c && sc >= tab {
        b';'
    } else if tab > c {
        b'\t'
    } else {
        b','
    }
}

fn rows_to_items(grid: Vec<Vec<String>>, default_unit: Unit) -> Result<Vec<ImportRow>, String> {
    let header_idx = grid
        .iter()
        .position(|r| r.iter().any(|c| !c.trim().is_empty()))
        .ok_or_else(|| "fichier vide".to_string())?;
    let header: Vec<String> = grid[header_idx].iter().map(|h| normalize(h)).collect();

    let find = |cands: &[&str]| header.iter().position(|h| cands.contains(&h.as_str()));

    let col_len = find(&["longueur", "length", "long", "len", "dimension", "longeur"])
        .ok_or_else(|| "colonne 'longueur' introuvable dans l'entete".to_string())?;
    let col_part = find(&["partid", "piece", "part", "idpiece", "item", "no"]);
    let col_job = find(&[
        "jobid", "job", "numerodejob", "numero", "nojob", "jobnumber", "ndejob", "nodejob",
    ]);
    let col_model = find(&["model", "modele", "profil", "profile"]);
    let col_qty = find(&["quantite", "qty", "quantity", "qte", "nb", "nombre"]);
    let col_unit = find(&["unite", "unit", "unites", "units"]);

    let mut out = Vec::new();
    for (i, row) in grid.iter().enumerate().skip(header_idx + 1) {
        let get = |c: Option<usize>| -> Option<String> {
            c.and_then(|idx| row.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };

        let len_raw = match get(Some(col_len)) {
            Some(v) => v,
            None => continue, // ligne sans longueur -> ignoree
        };

        let row_unit = get(col_unit)
            .and_then(|u| Unit::from_str(&u))
            .unwrap_or(default_unit);
        let length_um = parse_length(&len_raw, row_unit)
            .ok_or_else(|| format!("longueur invalide a la ligne {} : « {len_raw} »", i + 1))?;
        if length_um <= 0 {
            return Err(format!("longueur nulle ou negative ligne {}", i + 1));
        }

        let qty = get(col_qty)
            .and_then(|s| s.replace(',', ".").parse::<f64>().ok())
            .map(|f| f.round().max(1.0) as u32)
            .unwrap_or(1);

        let part_id = get(col_part).unwrap_or_else(|| format!("P{}", out.len() + 1));

        out.push(ImportRow {
            job_id: get(col_job),
            part_id,
            model: get(col_model),
            length_raw: len_raw,
            length_um,
            qty,
        });
    }

    if out.is_empty() {
        return Err("aucune ligne de coupe valide trouvee".to_string());
    }
    Ok(out)
}

/// Normalise un entete : minuscules, sans accents, sans espaces/underscores/#.
fn normalize(s: &str) -> String {
    let mut out = String::new();
    for ch in s.trim().to_lowercase().chars() {
        let mapped = match ch {
            'à' | 'â' | 'ä' | 'á' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ç' => 'c',
            c => c,
        };
        if mapped.is_ascii_alphanumeric() {
            out.push(mapped);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::units::foot;

    #[test]
    fn import_csv_de_base() {
        let csv = "jobID,partID,model,longueur,quantite\n\
                   JOB-1,P1,U-100,6,2\n\
                   JOB-1,P2,U-100,5,6\n";
        let rows = parse_file(csv.as_bytes(), FileKind::Csv, Unit::Foot).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].part_id, "P1");
        assert_eq!(rows[0].qty, 2);
        assert_eq!(rows[0].length_um, foot(6.0));
        assert_eq!(rows[0].model.as_deref(), Some("U-100"));
        assert_eq!(rows[1].qty, 6);
    }

    #[test]
    fn import_csv_point_virgule_et_sans_quantite() {
        let csv = "partID;longueur\nP1;5\nP2;3\n";
        let rows = parse_file(csv.as_bytes(), FileKind::Csv, Unit::Foot).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].qty, 1); // pas de colonne quantite -> 1
        assert_eq!(rows[1].length_um, foot(3.0));
    }

    #[test]
    fn entete_avec_accents_et_unite() {
        let csv = "Numéro de job,Pièce,Modèle,Longueur,Quantité,Unité\n\
                   J9,A,M1,60,1,po\n";
        let rows = parse_file(csv.as_bytes(), FileKind::Csv, Unit::Foot).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].job_id.as_deref(), Some("J9"));
        // 60 po (et non 60 pi) grace a la colonne unite
        assert_eq!(rows[0].length_um, crate::optimizer::units::inch(60.0));
    }
}
