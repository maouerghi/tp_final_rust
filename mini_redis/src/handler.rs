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

/// Traite une requête et retourne une réponse.
async fn process_request(line: &str, store: SharedStore) -> Response {
    // Parser la requête
    let req = match parse_request(line) {
        Ok(r) => r,
        Err(e) => {
            debug!("Parse error: {}", e);
            return Response::error(e);
        }
    };

    // Parser la commande
    let cmd = match Command::from_str(&req.cmd) {
        Some(c) => c,
        None => {
            debug!("Unknown command: {}", req.cmd);
            return Response::error("unknown command");
        }
    };

    // Exécuter la commande
    execute_command(cmd, req, store).await
}

/// Exécute une commande spécifique.
async fn execute_command(
    cmd: Command,
    req: crate::command::Request,
    store: SharedStore,
) -> Response {
    match cmd {
        Command::Ping => {
            debug!("PING");
            Response::ok()
        }

        Command::Set => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };
            let value = match req.value {
                Some(v) => v,
                None => return Response::error("missing value"),
            };

            debug!("SET {} = {}", key, value);

            let mut store_guard = store.lock().await;
            store_guard.insert(key, Entry::new(value));

            Response::ok()
        }

        Command::Get => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };

            debug!("GET {}", key);

            let store_guard = store.lock().await;
            let response = match store_guard.get(&key) {
                Some(entry) => {
                    if entry.is_expired() {
                        Response::ok().with_value(serde_json::Value::Null)
                    } else {
                        Response::ok().with_value(serde_json::Value::String(entry.value.clone()))
                    }
                }
                None => Response::ok().with_value(serde_json::Value::Null),
            };

            response
        }

        Command::Del => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };

            debug!("DEL {}", key);

            let mut store_guard = store.lock().await;
            let count = if store_guard.remove(&key).is_some() { 1 } else { 0 };

            Response::ok().with_count(count)
        }

        Command::Keys => {
            debug!("KEYS");

            let store_guard = store.lock().await;
            let mut keys: Vec<String> = store_guard
                .iter()
                .filter(|(_, entry)| !entry.is_expired())
                .map(|(k, _)| k.clone())
                .collect();

            keys.sort(); // Pour plus de prévisibilité

            Response::ok().with_keys(keys)
        }

        Command::Expire => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };
            let seconds = match req.seconds {
                Some(s) => {
                    if s < 0 {
                        return Response::error("seconds must be positive");
                    }
                    s
                }
                None => return Response::error("missing seconds"),
            };

            debug!("EXPIRE {} {}", key, seconds);

            let mut store_guard = store.lock().await;
            if let Some(entry) = store_guard.get_mut(&key) {
                let expires_at = Instant::now() + std::time::Duration::from_secs(seconds as u64);
                entry.expires_at = Some(expires_at);
                Response::ok()
            } else {
                // La clé n'existe pas, on retourne ok quand même pour simplifier
                Response::ok()
            }
        }

        Command::Ttl => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };

            debug!("TTL {}", key);

            let store_guard = store.lock().await;
            let ttl = match store_guard.get(&key) {
                Some(entry) => {
                    if entry.is_expired() {
                        -2
                    } else {
                        entry.ttl()
                    }
                }
                None => -2,
            };

            Response::ok().with_ttl(ttl)
        }

        Command::Incr => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };

            debug!("INCR {}", key);

            let mut store_guard = store.lock().await;
            let new_value = match store_guard.get(&key) {
                Some(entry) => {
                    // Parser la valeur en entier
                    match entry.value.parse::<i64>() {
                        Ok(n) => (n + 1).to_string(),
                        Err(_) => return Response::error("not an integer"),
                    }
                }
                None => "1".to_string(),
            };

            let result_value = new_value.parse::<i64>().unwrap();
            store_guard.insert(key, Entry::new(new_value));

            Response::ok().with_value(serde_json::Value::Number(result_value.into()))
        }

        Command::Decr => {
            let key = match req.key {
                Some(k) => k,
                None => return Response::error("missing key"),
            };

            debug!("DECR {}", key);

            let mut store_guard = store.lock().await;
            let new_value = match store_guard.get(&key) {
                Some(entry) => {
                    // Parser la valeur en entier
                    match entry.value.parse::<i64>() {
                        Ok(n) => (n - 1).to_string(),
                        Err(_) => return Response::error("not an integer"),
                    }
                }
                None => "-1".to_string(),
            };

            let result_value = new_value.parse::<i64>().unwrap();
            store_guard.insert(key, Entry::new(new_value));

            Response::ok().with_value(serde_json::Value::Number(result_value.into()))
        }

        Command::Save => {
            debug!("SAVE");

            let store_guard = store.lock().await;

            // Créer un objet JSON simple
            let mut json_obj = serde_json::json!({});
            for (k, v) in store_guard.iter() {
                if !v.is_expired() {
                    json_obj[k] = serde_json::Value::String(v.value.clone());
                }
            }

            // Écrire dans dump.json
            match std::fs::write("dump.json", json_obj.to_string()) {
                Ok(_) => {
                    info!("Saved {} keys to dump.json", store_guard.len());
                    Response::ok()
                }
                Err(e) => {
                    warn!("Failed to save dump.json: {}", e);
                    Response::error(format!("save failed: {}", e))
                }
            }
        }
    }
}