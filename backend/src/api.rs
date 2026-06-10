//! Types de requete/reponse de l'API HTTP.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::drilling::DrillResult;
use crate::optimizer::{PartType, Solution};

/// Reponse de l'import de fichier.
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub count: usize,
    /// Numeros de job distincts rencontres dans la colonne jobID (le cas echeant).
    pub job_numbers: Vec<String>,
    pub items: Vec<ImportItem>,
}

#[derive(Debug, Serialize)]
pub struct ImportItem {
    pub part_id: String,
    pub model: Option<String>,
    pub job_id: Option<String>,
    pub length_um: i64,
    pub qty: u32,
}

/// Requete d'export CSV du plan de coupe (sans passer par la base).
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    #[serde(default)]
    pub job_number: String,
    pub parts: Vec<PartType>,
    pub solution: Solution,
    /// "pi" | "po" | "mm"
    #[serde(default = "default_unit")]
    pub unit: String,
}

fn default_unit() -> String {
    "pi".to_string()
}

/// Requete d'enregistrement d'un job en base.
#[derive(Debug, Deserialize)]
pub struct SaveJobRequest {
    pub job_number: String,
    #[serde(default)]
    pub status: Option<String>,
    pub settings: Value,
    pub stocks: Value,
    pub parts: Value,
    #[serde(default)]
    pub result: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct SaveJobResponse {
    pub id: String,
}

/// Requete de calcul de percage (parametres globaux appliques a chaque piece).
#[derive(Debug, Deserialize)]
pub struct DrillingRequest {
    pub parts: Vec<PartType>,
    pub pas_um: i64,
    pub marge_min_um: i64,
    pub marge_max_um: i64,
    /// "pi" | "po" | "mm" — utilise uniquement pour l'export CSV.
    #[serde(default = "default_unit")]
    pub unit: String,
}

/// Repartition de trous pour une piece donnee.
#[derive(Debug, Serialize)]
pub struct PartDrilling {
    pub part_id: String,
    pub label: String,
    pub model: Option<String>,
    pub length_um: i64,
    pub qty: u32,
    pub result: DrillResult,
}

#[derive(Debug, Serialize)]
pub struct DrillingResponse {
    pub results: Vec<PartDrilling>,
}
