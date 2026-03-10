//! Module de gestion des erreurs.
//!
//! Fournit des types d'erreurs typés pour le serveur MiniRedis,
//! permettant une gestion cohérente et une sérialisation JSON correcte.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Erreur générique du serveur MiniRedis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerError {
    pub message: String,
}

