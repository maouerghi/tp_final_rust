//! Module de gestion des clients.
//!
//! Traite chaque connexion client, lit les requêtes JSON ligne par ligne,
//! exécute les commandes et envoie les réponses.

use crate::command::{parse_request, Command, Response};
use crate::store::{Entry, SharedStore};
use std::net::SocketAddr;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

/// Traite une connexion client.
///
/// Lit les requêtes JSON ligne par ligne, exécute les commandes,
/// et envoie les réponses.
pub async fn handle_client(
    socket: TcpStream,
    store: SharedStore,
    addr: SocketAddr,
) {
    info!("New client connected: {}", addr);

    let (read, mut write) = socket.into_split();
    let mut reader = BufReader::new(read);
    let mut line = String::new();

    loop {
        // Réinitialiser le buffer de ligne
        line.clear();

        // Lire une ligne
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF : client fermé
                info!("Client disconnected: {}", addr);
                break;
            }
            Ok(_) => {
                // Ligne reçue
                let response = process_request(&line.trim(), store.clone()).await;
                let response_json = response.to_json_line();

                // Envoyer la réponse
                if let Err(e) = write.write_all(response_json.as_bytes()).await {
                    error!("Failed to write response to {}: {}", addr, e);
                    break;
                }
                if let Err(e) = write.write_all(b"\n").await {
                    error!("Failed to write newline to {}: {}", addr, e);
                    break;
                }
            }
            Err(e) => {
                error!("Failed to read from {}: {}", addr, e);
                break;
            }
        }
    }

    info!("Handler finished for {}", addr);
}