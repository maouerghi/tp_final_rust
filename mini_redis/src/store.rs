//! Module de gestion du store partagé.
//!
//! Gère le stockage en mémoire des paires clé/valeur avec support des expirations.
//! Utilise `Arc<Mutex<HashMap>>` pour la sécurité thread-safe.


use serde::{Deserialize, Serialize};

/// Représente une requête du client.
/// Champ `cmd` obligatoire, autres champs dépendent de la commande.
#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,

    #[serde(default)]
    pub key: Option<String>,

    #[serde(default)]
    pub value: Option<String>,

    #[serde(default)]
    pub seconds: Option<i64>,
}

/// Réponse du serveur (succès ou erreur).
#[derive(Debug, Serialize)]
pub struct Response {
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
