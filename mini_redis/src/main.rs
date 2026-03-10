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
async fn main() {
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
    // TODO: Implémenter le serveur MiniRedis sur 127.0.0.1:7878
    //
    // Étapes suggérées :
    // 1. Créer le store partagé (Arc<Mutex<HashMap<String, ...>>>)
    // 2. Bind un TcpListener sur 127.0.0.1:7878
    // 3. Accept loop : pour chaque connexion, spawn une tâche
    // 4. Dans chaque tâche : lire les requêtes JSON ligne par ligne,
    //    traiter la commande, envoyer la réponse JSON + '\n'

    println!("MiniRedis - à implémenter !");
}
