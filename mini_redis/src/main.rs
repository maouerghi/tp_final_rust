//! # MiniRedis - Serveur Key-Value Asynchrone
//!
//! Un serveur Redis minimaliste implémenté en Rust avec Tokio.
//! Supports les commandes : PING, SET, GET, DEL, KEYS, EXPIRE, TTL, INCR, DECR, SAVE
//!
//! ## Protocole
//! - **Transport** : TCP sur `127.0.0.1:7878`
//! - **Format** : Requêtes/réponses JSON, une par ligne, terminées par `\n`
//!
//! ## Architecture
//! - `store.rs` : Stockage thread-safe avec expirations
//! - `command.rs` : Types de requête/réponse
//! - `handler.rs` : Traitement des clients TCP
//! - `expiry.rs` : Nettoyage des clés expirées en arrière-plan
//! - `error.rs` : Gestion des erreurs

mod command;
mod error;
mod expiry;
mod handler;
mod store;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // === INITIALISATION DU LOGGING ===
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting MiniRedis server...");

    // === SETUP DU SERVEUR ===
    let addr: SocketAddr = "127.0.0.1:7878".parse()?;
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    // === CRÉATION DU STORE PARTAGÉ ===
    let store = store::new_shared_store();
    info!("Shared store initialized");

    // === LANCEMENT DU NETTOYAGE DES EXPIRATIONS ===
    let _cleanup_handle = expiry::spawn_expiry_cleanup(store.clone());
    info!("Expiry cleanup task spawned");

    // === ACCEPT LOOP ===
    info!("Accepting connections...");
    loop {
        let (socket, addr) = listener.accept().await?;
        let store = store.clone();

        // Spawn une tâche pour chaque client
        tokio::spawn(async move {
            handler::handle_client(socket, store, addr).await;
        });
    }

    // Note: le cleanup_handle n'est jamais await-é car la boucle est infinie.
    // En pratique, il s'exécutera jusqu'à l'arrêt du serveur.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        // Test simple pour vérifier que le projet compile
        assert_eq!("MiniRedis", "MiniRedis");
    }
}
