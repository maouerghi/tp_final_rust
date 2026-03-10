//! Module de gestion du nettoyage des expirations.
//!
//! Lance une tâche de fond qui nettoie périodiquement les clés expirées
//! du store.

use crate::store::SharedStore;
use std::time::Duration;
use tracing::{debug, info};

/// Lance une tâche de nettoyage des clés expirées.
///
/// Cette tâche s'exécute toutes les secondes et supprime les clés
/// dont la date d'expiration a dépassé maintenant.
pub fn spawn_expiry_cleanup(store: SharedStore) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        info!("Expiry cleanup task started");

        loop {
            interval.tick().await;

            // Acquérir le verrou
            let mut store_guard = store.lock().await;

            // Nombre de clés supprimées dans ce cycle
            let initial_count = store_guard.len();

            // Supprimer les clés expirées
            store_guard.retain(|key, entry| {
                let keep = !entry.is_expired();
                if !keep {
                    debug!("Removing expired key: {}", key);
                }
                keep
            });

            let removed = initial_count - store_guard.len();
            if removed > 0 {
                debug!("Cleaned {} expired keys", removed);
            }

            // Libérer le verrou explicitement
            drop(store_guard);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{new_shared_store, Entry};
    use std::time::Instant;

    #[tokio::test]
    async fn test_expiry_cleanup() {
        let store = new_shared_store();

        // Insérer une clé valide
        {
            let mut guard = store.lock().await;
            guard.insert("valid".to_string(), Entry::new("value".to_string()));
        }

        // Insérer une clé expirée
        {
            let mut guard = store.lock().await;
            let past = Instant::now() - Duration::from_secs(1);
            guard.insert(
                "expired".to_string(),
                Entry::with_expiry("old".to_string(), past),
            );
        }

        // Vérifier l'état initial
        {
            let guard = store.lock().await;
            assert_eq!(guard.len(), 2);
        }

        // Spawner le cleanup task
        let cleanup_handle = spawn_expiry_cleanup(store.clone());

        // Attendre que le cleanup s'exécute
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Vérifier que la clé expirée a été supprimée
        {
            let guard = store.lock().await;
            assert_eq!(guard.len(), 1);
            assert!(guard.contains_key("valid"));
            assert!(!guard.contains_key("expired"));
        }

        // Arrêter le cleanup task
        cleanup_handle.abort();
    }
}
