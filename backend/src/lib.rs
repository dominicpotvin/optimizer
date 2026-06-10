// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
//! Bibliotheque de l'optimiseur de coupe.
//!
//! Le binaire (`main.rs`) expose ce moteur via une API HTTP (Axum) + PostgreSQL.

pub mod api;
pub mod db;
pub mod drilling;
pub mod export;
pub mod import;
pub mod optimizer;
pub mod parse;

pub use optimizer::{solve, Problem, Solution};
