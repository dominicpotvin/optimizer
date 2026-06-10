// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Persistance PostgreSQL (sqlx) : enregistrement et historique des jobs.
//!
//! Schema volontairement simple : un seul tableau `jobs` avec les longueurs de
//! stock, la liste de coupe, les parametres et le resultat stockes en JSONB.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub type Db = PgPool;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS jobs (
    id          UUID PRIMARY KEY,
    job_number  TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    status      TEXT NOT NULL DEFAULT 'enregistre',
    settings    JSONB NOT NULL,
    stocks      JSONB NOT NULL,
    parts       JSONB NOT NULL,
    result      JSONB
);
CREATE INDEX IF NOT EXISTS idx_jobs_created ON jobs (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_number  ON jobs (job_number);
"#;

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct JobSummary {
    pub id: Uuid,
    pub job_number: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub total_bars: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct JobRecord {
    pub id: Uuid,
    pub job_number: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub settings: Value,
    pub stocks: Value,
    pub parts: Value,
    pub result: Option<Value>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_job(
    pool: &PgPool,
    job_number: &str,
    status: &str,
    settings: &Value,
    stocks: &Value,
    parts: &Value,
    result: Option<&Value>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO jobs (id, job_number, status, settings, stocks, parts, result)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(job_number)
    .bind(status)
    .bind(settings)
    .bind(stocks)
    .bind(parts)
    .bind(result)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn list_jobs(pool: &PgPool) -> Result<Vec<JobSummary>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, job_number, created_at, status,
                (result->'summary'->>'total_bars')::bigint AS total_bars
         FROM jobs
         ORDER BY created_at DESC
         LIMIT 200",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| JobSummary {
            id: r.get("id"),
            job_number: r.get("job_number"),
            created_at: r.get("created_at"),
            status: r.get("status"),
            total_bars: r.try_get("total_bars").ok(),
        })
        .collect())
}

pub async fn get_job(pool: &PgPool, id: Uuid) -> Result<Option<JobRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, job_number, created_at, status, settings, stocks, parts, result
         FROM jobs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| JobRecord {
        id: r.get("id"),
        job_number: r.get("job_number"),
        created_at: r.get("created_at"),
        status: r.get("status"),
        settings: r.get("settings"),
        stocks: r.get("stocks"),
        parts: r.get("parts"),
        result: r.get("result"),
    }))
}

pub async fn delete_job(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM jobs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
