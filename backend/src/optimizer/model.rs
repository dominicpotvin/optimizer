// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Types du domaine pour l'optimiseur de coupe (1D cutting stock).
//!
//! Toutes les longueurs sont exprimees en **micrometres (i64)** afin de
//! travailler en arithmetique entiere exacte (aucune erreur d'arrondi sur
//! les fractions imperiales du type 1/8", 1/16", etc.). La conversion
//! depuis/vers pi-po ou mm est faite cote frontend (ou via le module `units`).

use serde::{Deserialize, Serialize};

/// Une longueur de barre brute disponible a l'inventaire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockType {
    pub id: String,
    /// Etiquette lisible (ex. "16 pi", "6096 mm").
    pub label: String,
    /// Longueur de la barre brute, en micrometres.
    pub length_um: i64,
    /// Quantite disponible. `None` = illimitee.
    #[serde(default)]
    pub available: Option<u32>,
}

/// Une piece a couper, avec sa quantite demandee.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartType {
    pub id: String,
    pub label: String,
    /// Longueur de la piece finie, en micrometres.
    pub length_um: i64,
    pub qty: u32,
    /// Modele / profil de reference (colonne "model" de l'import). Optionnel.
    #[serde(default)]
    pub model: Option<String>,
}

/// Parametres globaux de l'optimisation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Largeur du trait de scie (kerf), en micrometres.
    pub kerf_um: i64,
    /// Une chute >= ce seuil est consideree reutilisable (retour stock),
    /// donc PAS comptee comme perte. En micrometres.
    #[serde(default)]
    pub reusable_threshold_um: i64,
    /// Budget de temps pour la recherche locale, en millisecondes.
    #[serde(default)]
    pub time_limit_ms: Option<u64>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            kerf_um: 0,
            reusable_threshold_um: 0,
            time_limit_ms: None,
        }
    }
}

/// Probleme complet a resoudre.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Problem {
    pub stocks: Vec<StockType>,
    pub parts: Vec<PartType>,
    pub settings: Settings,
}

/// Une coupe placee sur une barre.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlacedCut {
    pub part_id: String,
    pub label: String,
    pub length_um: i64,
}

/// Le plan de coupe d'une seule barre brute.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BarPlan {
    pub stock_id: String,
    pub stock_label: String,
    pub stock_length_um: i64,
    pub cuts: Vec<PlacedCut>,
    /// Somme des longueurs des pieces (sans le kerf).
    pub used_um: i64,
    /// Matiere consommee par les traits de scie.
    pub kerf_total_um: i64,
    /// Reste de barre apres la derniere coupe.
    pub offcut_um: i64,
    /// `true` si `offcut_um >= reusable_threshold_um`.
    pub reusable: bool,
}

impl BarPlan {
    pub fn new(stock: &StockType) -> Self {
        BarPlan {
            stock_id: stock.id.clone(),
            stock_label: stock.label.clone(),
            stock_length_um: stock.length_um,
            cuts: Vec::new(),
            used_um: 0,
            kerf_total_um: 0,
            offcut_um: stock.length_um,
            reusable: false,
        }
    }

    /// Matiere totale engagee (pieces + traits de scie).
    pub fn consumed_um(&self) -> i64 {
        self.used_um + self.kerf_total_um
    }

    /// Espace restant disponible sur la barre.
    pub fn remaining_um(&self) -> i64 {
        self.stock_length_um - self.consumed_um()
    }

    /// Peut-on ajouter une piece de `len_um` (en payant un kerf de plus) ?
    pub fn can_fit(&self, len_um: i64, kerf_um: i64) -> bool {
        len_um + kerf_um <= self.remaining_um()
    }

    /// Ajoute une piece (suppose `can_fit` deja verifie par l'appelant).
    pub fn push(&mut self, cut: PlacedCut, kerf_um: i64) {
        self.used_um += cut.length_um;
        self.kerf_total_um += kerf_um;
        self.cuts.push(cut);
        self.offcut_um = self.remaining_um();
    }

    /// Change le type de stock de la barre (les coupes restent identiques).
    /// Recalcule la chute. Suppose que `stock.length_um >= consumed_um()`.
    pub fn set_stock(&mut self, stock: &StockType) {
        self.stock_id = stock.id.clone();
        self.stock_label = stock.label.clone();
        self.stock_length_um = stock.length_um;
        self.offcut_um = self.remaining_um();
    }

    pub fn finalize(&mut self, reusable_threshold_um: i64) {
        self.offcut_um = self.remaining_um();
        self.reusable =
            reusable_threshold_um > 0 && self.offcut_um >= reusable_threshold_um;
    }
}

/// Resume chiffre d'une solution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_bars: usize,
    /// Nombre de barres utilisees par type de stock (label -> compte).
    pub bars_by_stock: Vec<StockUsage>,
    /// Longueur brute totale engagee (somme des barres utilisees).
    pub total_stock_um: i64,
    /// Longueur totale des pieces produites.
    pub total_parts_um: i64,
    /// Matiere consommee par les traits de scie.
    pub total_kerf_um: i64,
    /// Somme de toutes les chutes (reutilisables + perte).
    pub total_offcut_um: i64,
    /// Somme des chutes >= seuil (recuperables).
    pub reusable_offcut_um: i64,
    /// Somme des chutes < seuil (perte reelle).
    pub real_waste_um: i64,
    /// Taux d'utilisation: pieces / brut, en pourcentage (0-100).
    pub utilization_pct: f64,
    /// Nombre de chutes reutilisables.
    pub reusable_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockUsage {
    pub stock_id: String,
    pub label: String,
    pub length_um: i64,
    pub count: usize,
}

/// Solution complete renvoyee au client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Solution {
    pub bars: Vec<BarPlan>,
    pub summary: Summary,
    /// `true` si toutes les pieces demandees ont pu etre placees.
    pub complete: bool,
    /// Pieces qui n'ont pas pu etre placees (dispo de stock insuffisante,
    /// ou piece plus longue que toute barre disponible).
    pub unplaced: Vec<PlacedCut>,
}

impl Solution {
    /// Cout a minimiser: matiere reellement perdue (perte reelle + kerf).
    /// Plus c'est bas, mieux c'est. Les chutes reutilisables ne comptent pas.
    pub fn cost_um(&self) -> i64 {
        self.summary.real_waste_um + self.summary.total_kerf_um
    }
}
