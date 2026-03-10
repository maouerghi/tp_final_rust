//! Module de gestion des erreurs.
//!
//! Fournit des types d'erreurs typés pour le serveur MiniRedis,
//! permettant une gestion cohérente et une sérialisation JSON correcte.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Erreur générique du serveur MiniRedis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ServerError {
    pub message: String,
}
#[allow(dead_code)]
impl ServerError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }

    pub fn invalid_json() -> Self {
        Self::new("invalid json")
    }

    pub fn unknown_command() -> Self {
        Self::new("unknown command")
    }

    pub fn not_an_integer() -> Self {
        Self::new("not an integer")
    }

    pub fn invalid_key() -> Self {
        Self::new("invalid key")
    }

    pub fn io_error(e: std::io::Error) -> Self {
        Self::new(format!("io error: {}", e))
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ServerError {}
