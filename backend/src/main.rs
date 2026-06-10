// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Serveur HTTP de l'optimiseur de coupe (Axum + PostgreSQL).
//!
//! Endpoints :
//!   POST   /api/optimize        -> calcule un plan de coupe (Problem -> Solution)
//!   POST   /api/import          -> multipart (file=.xlsx/.csv, unit) -> items
//!   GET    /api/template.csv    -> modele d'import a remplir
//!   POST   /api/export.csv      -> export CSV d'un plan de coupe
//!   POST   /api/jobs            -> enregistre un job (DB)
//!   GET    /api/jobs            -> historique des jobs (DB)
//!   GET    /api/jobs/:id        -> un job (DB)
//!   DELETE /api/jobs/:id        -> supprime un job (DB)
//!   GET    /api/health          -> sonde
//!   *                           -> build statique React (fallback SPA)
//!
//! Variables d'environnement : PORT (8080), STATIC_DIR (./static),
//! DATABASE_URL (optionnelle ; sans elle, /api/jobs renvoie 503).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use axum::{
    extract::{Json, Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use optimiseur_coupe::api::{
    DrillingRequest, DrillingResponse, ExportRequest, ImportItem, ImportResponse, PartDrilling,
    SaveJobRequest, SaveJobResponse,
};
use optimiseur_coupe::optimizer::{self, PartType, Problem};
use optimiseur_coupe::parse::Unit;
use optimiseur_coupe::{db, drilling, export, import};

#[derive(Clone)]
struct AppState {
    pool: Option<PgPool>,
}

const BOM: &str = "\u{FEFF}"; // pour qu'Excel lise bien l'UTF-8 (accents)

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string());

    let pool = match std::env::var("DATABASE_URL") {
        Ok(url) => init_db(&url).await,
        Err(_) => {
            tracing::warn!("DATABASE_URL absente : persistance desactivee (/api/jobs -> 503)");
            None
        }
    };

    let state = AppState { pool };

    let static_path = PathBuf::from(&static_dir);
    let serve_dir = ServeDir::new(&static_path)
        .not_found_service(ServeFile::new(static_path.join("index.html")));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/optimize", post(optimize))
        .route("/api/import", post(import_file))
        .route("/api/template.csv", get(template_csv))
        .route("/api/export.csv", post(export_csv))
        .route("/api/drilling", post(drilling_handler))
        .route("/api/drilling.csv", post(drilling_csv))
        .route("/api/jobs", post(save_job).get(list_jobs))
        .route("/api/jobs/:id", get(get_job).delete(delete_job))
        .fallback_service(serve_dir)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("optimiseur-coupe en ecoute sur http://{addr} (static: {static_dir})");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("impossible de lier le port");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("erreur serveur");
}

async fn init_db(url: &str) -> Option<PgPool> {
    for attempt in 1..=20 {
        match db::connect(url).await {
            Ok(pool) => {
                if let Err(e) = db::migrate(&pool).await {
                    tracing::warn!("migration echouee (tentative {attempt}) : {e}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                tracing::info!("PostgreSQL connecte et schema pret");
                return Some(pool);
            }
            Err(e) => {
                tracing::warn!("connexion PostgreSQL impossible (tentative {attempt}) : {e}");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    tracing::error!("abandon de la connexion PostgreSQL ; /api/jobs sera indisponible");
    None
}

fn bad(msg: impl Into<String>) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg.into() }))).into_response()
}

fn no_db() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "base de donnees non configuree" })),
    )
        .into_response()
}

fn csv_download(filename: &str, body: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        format!("{BOM}{body}"),
    )
        .into_response()
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn optimize(Json(problem): Json<Problem>) -> Response {
    if problem.stocks.is_empty() {
        return bad("aucune longueur de stock fournie");
    }
    if problem.stocks.iter().any(|s| s.length_um <= 0)
        || problem.parts.iter().any(|p| p.length_um <= 0)
    {
        return bad("longueurs invalides (doivent etre > 0)");
    }
    if problem.settings.kerf_um < 0 {
        return bad("kerf negatif");
    }
    let solution = optimizer::solve(&problem);
    (StatusCode::OK, Json(solution)).into_response()
}

async fn import_file(mut mp: Multipart) -> Response {
    let mut bytes: Option<Vec<u8>> = None;
    let mut filename = String::new();
    let mut unit = Unit::Foot;

    loop {
        let field = match mp.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return bad(format!("requete multipart invalide : {e}")),
        };
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().unwrap_or("").to_string();
                match field.bytes().await {
                    Ok(b) => bytes = Some(b.to_vec()),
                    Err(e) => return bad(format!("lecture du fichier impossible : {e}")),
                }
            }
            "unit" => {
                if let Ok(t) = field.text().await {
                    if let Some(u) = Unit::from_str(&t) {
                        unit = u;
                    }
                }
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let bytes = match bytes {
        Some(b) => b,
        None => return bad("aucun champ 'file' dans la requete"),
    };

    let kind = import::detect_kind(&filename);
    match import::parse_file(&bytes, kind, unit) {
        Ok(rows) => {
            let mut job_numbers: Vec<String> = rows
                .iter()
                .filter_map(|r| r.job_id.clone())
                .collect();
            job_numbers.sort();
            job_numbers.dedup();

            let items: Vec<ImportItem> = rows
                .into_iter()
                .map(|r| ImportItem {
                    part_id: r.part_id,
                    model: r.model,
                    job_id: r.job_id,
                    length_um: r.length_um,
                    qty: r.qty,
                })
                .collect();

            Json(ImportResponse {
                count: items.len(),
                job_numbers,
                items,
            })
            .into_response()
        }
        Err(e) => bad(e),
    }
}

async fn template_csv() -> Response {
    csv_download("modele_liste_coupe.csv", export::template_csv())
}

async fn export_csv(Json(req): Json<ExportRequest>) -> Response {
    let unit = Unit::from_str(&req.unit).unwrap_or(Unit::Foot);
    let body = export::solution_to_csv(&req.job_number, &req.solution, &req.parts, unit);
    let fname = if req.job_number.is_empty() {
        "plan_de_coupe.csv".to_string()
    } else {
        format!("plan_de_coupe_{}.csv", sanitize(&req.job_number))
    };
    csv_download(&fname, body)
}

fn compute_drilling(parts: &[PartType], params: &drilling::DrillingParams) -> Vec<PartDrilling> {
    parts
        .iter()
        .map(|p| PartDrilling {
            part_id: p.id.clone(),
            label: p.label.clone(),
            model: p.model.clone(),
            length_um: p.length_um,
            qty: p.qty,
            result: drilling::compute(p.length_um, params),
        })
        .collect()
}

async fn drilling_handler(Json(req): Json<DrillingRequest>) -> Response {
    if req.pas_um <= 0 {
        return bad("entraxe (pas) invalide");
    }
    let params = drilling::DrillingParams {
        pas_um: req.pas_um,
        marge_min_um: req.marge_min_um.max(0),
        marge_max_um: req.marge_max_um,
    };
    let results = compute_drilling(&req.parts, &params);
    Json(DrillingResponse { results }).into_response()
}

async fn drilling_csv(Json(req): Json<DrillingRequest>) -> Response {
    if req.pas_um <= 0 {
        return bad("entraxe (pas) invalide");
    }
    let unit = Unit::from_str(&req.unit).unwrap_or(Unit::Mm);
    let params = drilling::DrillingParams {
        pas_um: req.pas_um,
        marge_min_um: req.marge_min_um.max(0),
        marge_max_um: req.marge_max_um,
    };
    let results = compute_drilling(&req.parts, &params);
    csv_download("plan_de_percage.csv", export::drilling_to_csv(&results, unit))
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

async fn save_job(State(state): State<AppState>, Json(req): Json<SaveJobRequest>) -> Response {
    let Some(pool) = state.pool else {
        return no_db();
    };
    let status = req.status.unwrap_or_else(|| "enregistre".to_string());
    match db::create_job(
        &pool,
        &req.job_number,
        &status,
        &req.settings,
        &req.stocks,
        &req.parts,
        req.result.as_ref(),
    )
    .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            Json(SaveJobResponse { id: id.to_string() }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("enregistrement impossible : {e}") })),
        )
            .into_response(),
    }
}

async fn list_jobs(State(state): State<AppState>) -> Response {
    let Some(pool) = state.pool else {
        return no_db();
    };
    match db::list_jobs(&pool).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}

async fn get_job(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let Some(pool) = state.pool else {
        return no_db();
    };
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad("identifiant invalide");
    };
    match db::get_job(&pool, uuid).await {
        Ok(Some(rec)) => Json(rec).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "job introuvable" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}

async fn delete_job(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let Some(pool) = state.pool else {
        return no_db();
    };
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad("identifiant invalide");
    };
    match db::delete_job(&pool, uuid).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "job introuvable" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("{e}") })),
        )
            .into_response(),
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("arret demande, fermeture...");
}
