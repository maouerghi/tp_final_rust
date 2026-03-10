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