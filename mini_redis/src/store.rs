//! Module de gestion du store partagé.
//!
//! Gère le stockage en mémoire des paires clé/valeur avec support des expirations.
//! Utilise `Arc<Mutex<HashMap>>` pour la sécurité thread-safe.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Entrée du store avec optionnellement une date d'expiration.
#[derive(Debug, Clone)]
pub struct Entry {
    /// La valeur stockée (String ou interprétée comme entier pour INCR/DECR)
    pub value: String,

    /// Instant d'expiration, None si pas d'expiration
    pub expires_at: Option<Instant>,
}

#[allow(dead_code)]
impl Entry {
    /// Crée une nouvelle entrée sans expiration.
    pub fn new(value: String) -> Self {
        Self {
            value,
            expires_at: None,
        }
    }

    /// Crée une nouvelle entrée avec expiration.
    pub fn with_expiry(value: String, expires_at: Instant) -> Self {
        Self {
            value,
            expires_at: Some(expires_at),
        }
    }

    /// Vérifie si l'entrée a expiré.
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Instant::now() >= exp)
            .unwrap_or(false)
    }

    /// Retourne le TTL en secondes, ou -1 si pas d'expiration, -2 si expiré.
    pub fn ttl(&self) -> i64 {
        if self.is_expired() {
            return -2;
        }

        match self.expires_at {
            Some(exp) => {
                // Calculer le temps restant jusqu'à l'expiration
                let remaining = exp.saturating_duration_since(Instant::now());

                // Si saturating_duration_since retourne 0, la clé a expiré
                if remaining.as_secs() == 0 && remaining.subsec_nanos() == 0 {
                    return -2;
                }

                // Retourner le TTL en secondes (arrondir à la hausse)
                let secs = remaining.as_secs();
                if remaining.subsec_nanos() > 0 {
                    (secs + 1) as i64
                } else {
                    secs as i64
                }
            }
            None => -1,
        }
    }
}

/// Type alias pour le store partagé thread-safe.
pub type SharedStore = Arc<Mutex<HashMap<String, Entry>>>;

/// Factory pour créer un nouveau store.
pub fn new_shared_store() -> SharedStore {
    Arc::new(Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_no_expiry() {
        let entry = Entry::new("value".to_string());
        assert!(!entry.is_expired());
        assert_eq!(entry.ttl(), -1);
    }

    #[test]
    fn test_entry_with_expiry() {
        let future = Instant::now() + std::time::Duration::from_secs(10);
        let entry = Entry::with_expiry("value".to_string(), future);
        assert!(!entry.is_expired());
        assert!(entry.ttl() > 0 && entry.ttl() <= 10);
    }

    #[test]
    fn test_entry_expired() {
        let past = Instant::now() - std::time::Duration::from_secs(1);
        let entry = Entry::with_expiry("value".to_string(), past);
        assert!(entry.is_expired());
        assert_eq!(entry.ttl(), -2);
    }
}
