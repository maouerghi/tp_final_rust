//! Module de gestion des commandes.
//!
//! Définit les types Request et Response sérialisables en JSON,
//! ainsi que les fonctions de parsing.

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

impl Response {
    /// Crée une réponse de succès.
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            value: None,
            count: None,
            keys: None,
            ttl: None,
            message: None,
        }
    }

    /// Crée une réponse d'erreur.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            value: None,
            count: None,
            keys: None,
            ttl: None,
            message: Some(message.into()),
        }
    }

    /// Ajoute une valeur à la réponse.
    pub fn with_value(mut self, value: impl Into<serde_json::Value>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Ajoute un count à la réponse.
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Ajoute une liste de clés à la réponse.
    pub fn with_keys(mut self, keys: Vec<String>) -> Self {
        self.keys = Some(keys);
        self
    }

    /// Ajoute un TTL à la réponse.
    pub fn with_ttl(mut self, ttl: i64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Convertit la réponse en JSON et l'ajoute à une ligne.
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            serde_json::to_string(&Response::error("serialization error")).unwrap()
        })
    }
}

/// Énumération des commandes supportées.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Ping,
    Set,
    Get,
    Del,
    Keys,
    Expire,
    Ttl,
    Incr,
    Decr,
    Save,
}

impl Command {
    /// Parse le nom de la commande depuis une string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PING" => Some(Command::Ping),
            "SET" => Some(Command::Set),
            "GET" => Some(Command::Get),
            "DEL" => Some(Command::Del),
            "KEYS" => Some(Command::Keys),
            "EXPIRE" => Some(Command::Expire),
            "TTL" => Some(Command::Ttl),
            "INCR" => Some(Command::Incr),
            "DECR" => Some(Command::Decr),
            "SAVE" => Some(Command::Save),
            _ => None,
        }
    }
}

/// Parse une ligne JSON en Request.
pub fn parse_request(line: &str) -> Result<Request, String> {
    serde_json::from_str(line).map_err(|_| "invalid json".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping() {
        let req = parse_request(r#"{"cmd": "PING"}"#).unwrap();
        assert_eq!(req.cmd, "PING");
    }

    #[test]
    fn test_parse_set() {
        let req = parse_request(r#"{"cmd": "SET", "key": "k", "value": "v"}"#).unwrap();
        assert_eq!(req.cmd, "SET");
        assert_eq!(req.key, Some("k".to_string()));
        assert_eq!(req.value, Some("v".to_string()));
    }

    #[test]
    fn test_parse_invalid_json() {
        let res = parse_request(r#"{"cmd": "PING""#);
        assert!(res.is_err());
    }

    #[test]
    fn test_response_ok() {
        let resp = Response::ok();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.message, None);
    }

    #[test]
    fn test_response_error() {
        let resp = Response::error("test error");
        assert_eq!(resp.status, "error");
        assert_eq!(resp.message, Some("test error".to_string()));
    }

    #[test]
    fn test_response_with_value() {
        let resp = Response::ok().with_value("hello");
        assert_eq!(resp.status, "ok");
        assert_eq!(
            resp.value,
            Some(serde_json::Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_command_parsing() {
        assert_eq!(Command::from_str("PING"), Some(Command::Ping));
        assert_eq!(Command::from_str("ping"), Some(Command::Ping));
        assert_eq!(Command::from_str("UNKNOWN"), None);
    }
}
