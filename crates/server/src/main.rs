use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};
use warp::Filter;
use futures::{StreamExt, SinkExt};
use proto::{SignalMessage, messages::SignalMessage::*};

struct Peer {
    sender: mpsc::UnboundedSender<Result<Message, warp::Error>>,
    #[allow(dead_code)]
    public_key: String,
    raw_id: String,
    brand: String,
}

type Peers = Arc<RwLock<HashMap<String, Peer>>>;
type Sessions = Arc<RwLock<HashMap<String, String>>>;

/// Normaliza cualquier ID eliminando espacios, guiones o puntos: "151 245 721" -> "151245721"
fn normalize_id(id: &str) -> String {
    id.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let peers: Peers = Arc::new(RwLock::new(HashMap::new()));
    let sessions: Sessions = Arc::new(RwLock::new(HashMap::new()));

    let peers_filter = warp::any().map(move || peers.clone());
    let sessions_filter = warp::any().map(move || sessions.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(peers_filter.clone())
        .and(sessions_filter.clone())
        .map(|ws: warp::ws::Ws, peers, sessions| {
            ws.on_upgrade(move |socket| handle_client(socket, peers, sessions))
        });

    let ws_root = warp::path::end()
        .and(warp::ws())
        .and(peers_filter.clone())
        .and(sessions_filter.clone())
        .map(|ws: warp::ws::Ws, peers, sessions| {
            ws.on_upgrade(move |socket| handle_client(socket, peers, sessions))
        });

    let health_route = warp::path("health")
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "online",
            "service": "RemoteDesktop-Rendezvous",
            "version": "1.3.0",
            "brands": ["wolfdesk", "farmadesk"]
        })));

    let routes = ws_route.or(ws_root).or(health_route);
    let port = 9050;

    println!("============================================================");
    println!(" 🚀 [Servidor de Señalización Multi-Organización v1.3]");
    println!(" Aislamiento de red activo: FarmaDesk (Farmanorte) & WolfDesk");
    println!(" Normalizador inteligente de IDs activo (con o sin espacios)");
    println!(" Escuchando conexiones WebSocket en ws://0.0.0.0:{}/ws", port);
    println!(" Endpoint de comprobación en http://localhost:{}/health", port);
    println!("============================================================");

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn handle_client(ws: WebSocket, peers: Peers, sessions: Sessions) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg?).await.is_err() {
                break;
            }
        }
        Ok::<(), warp::Error>(())
    });

    let mut current_norm_id: Option<String> = None;
    let mut current_raw_id: Option<String> = None;
    let mut current_brand: String = "wolfdesk".to_string();

    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Error en WebSocket: {}", e);
                break;
            }
        };

        if let Ok(text) = msg.to_str() {
            let parsed: Result<SignalMessage, _> = serde_json::from_str(text);
            match parsed {
                Ok(Register { client_id, public_key, brand }) => {
                    let client_brand = brand.unwrap_or_else(|| "wolfdesk".to_string()).to_lowercase();
                    current_brand = client_brand.clone();
                    let norm = normalize_id(&client_id);
                    println!("\n>> [{}] Cliente registrado: ID='{}' (Normalizado: '{}')", client_brand.to_uppercase(), client_id, norm);
                    current_norm_id = Some(norm.clone());
                    current_raw_id = Some(client_id.clone());

                    peers.write().await.insert(norm, Peer {
                        sender: tx.clone(),
                        public_key,
                        raw_id: client_id.clone(),
                        brand: client_brand,
                    });

                    let resp = RegisterSuccess { assigned_id: client_id };
                    let resp_json = serde_json::to_string(&resp).unwrap();
                    let _ = tx.send(Ok(Message::text(resp_json)));
                }

                Ok(ConnectRequest { target_id, password_hash, auth_signature }) => {
                    let from_raw = current_raw_id.clone().unwrap_or_else(|| "Desconocido".into());
                    let target_norm = normalize_id(&target_id);
                    let sender_brand = current_brand.clone();

                    println!(">> [{}] Solicitud de sesión: De '{}' hacia '{}' (Buscar: '{}')", sender_brand.to_uppercase(), from_raw, target_id, target_norm);

                    let read_guard = peers.read().await;
                    if let Some(target_peer) = read_guard.get(&target_norm) {
                        // Verificación de aislamiento estricto por marca / organización
                        if target_peer.brand != sender_brand {
                            println!(">> [SEGURIDAD] Bloqueada conexión cruzada: {} ({}) intentó conectar a {} ({})",
                                from_raw, sender_brand.to_uppercase(), target_peer.raw_id, target_peer.brand.to_uppercase());

                            let target_org = if target_peer.brand == "farmadesk" { "Farmanorte (FarmaDesk)" } else { "WolfDesk" };
                            let my_org = if sender_brand == "farmadesk" { "Farmanorte (FarmaDesk)" } else { "WolfDesk" };
                            let err = Error {
                                message: format!("Acceso restringido: El puesto [{}] pertenece a la red de {} y este equipo opera bajo {}. No se permiten conexiones cruzadas.", target_id, target_org, my_org),
                            };
                            let json = serde_json::to_string(&err).unwrap();
                            let _ = tx.send(Ok(Message::text(json)));
                        } else {
                            println!(">> [{}] ¡Puesto destino encontrado! Enrutando solicitud a '{}'", sender_brand.to_uppercase(), target_peer.raw_id);
                            let forward_msg = ConnectRequest {
                                target_id: from_raw,
                                password_hash,
                                auth_signature,
                            };
                            let json = serde_json::to_string(&forward_msg).unwrap();
                            let _ = target_peer.sender.send(Ok(Message::text(json)));
                        }
                    } else {
                        println!(">> [{}] Destino '{}' no encontrado en peers activos", sender_brand.to_uppercase(), target_norm);
                        let app_name = if sender_brand == "farmadesk" { "FarmaDesk" } else { "WolfDesk" };
                        let err = Error {
                            message: format!("El puesto {} [{}] no está en línea o no está conectado al servidor.", app_name, target_id),
                        };
                        let json = serde_json::to_string(&err).unwrap();
                        let _ = tx.send(Ok(Message::text(json)));
                    }
                }

                Ok(IncomingPrompt { from_id }) => {
                    forward_to_target(&peers, &from_id, IncomingPrompt { from_id: current_raw_id.clone().unwrap_or_default() }).await;
                }

                Ok(ConnectAccept { from_id, permissions }) => {
                    println!(">> [WOLFDESK] Sesión aceptada por '{}' con permisos: {:?}", from_id, permissions);
                    if let Some(ref my_norm) = current_norm_id {
                        let target_norm = normalize_id(&from_id);
                        sessions.write().await.insert(my_norm.clone(), target_norm.clone());
                        sessions.write().await.insert(target_norm, my_norm.clone());
                    }
                    forward_to_target(&peers, &from_id, ConnectAccept { from_id: current_raw_id.clone().unwrap_or_default(), permissions }).await;
                }

                Ok(ConnectReject { from_id, reason }) => {
                    println!(">> [WOLFDESK] Sesión rechazada por '{}': {}", from_id, reason);
                    forward_to_target(&peers, &from_id, ConnectReject { from_id: current_raw_id.clone().unwrap_or_default(), reason }).await;
                }

                Ok(Disconnect { target_id, reason }) => {
                    println!(">> [WOLFDESK] Sesión finalizada: {}", reason);
                    let target_norm = normalize_id(&target_id);
                    if let Some(ref my_norm) = current_norm_id {
                        sessions.write().await.remove(my_norm);
                        sessions.write().await.remove(&target_norm);
                    }
                    forward_to_target(&peers, &target_id, Disconnect { target_id: current_raw_id.clone().unwrap_or_default(), reason }).await;
                }

                Ok(Chat { target_id, text }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, Chat { target_id: from, text }).await;
                }

                Ok(FileTransferStart { target_id, file_name, file_size, total_chunks }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, FileTransferStart { target_id: from, file_name, file_size, total_chunks }).await;
                }

                Ok(FileTransferChunk { target_id, file_name, chunk_index, total_chunks, data_base64 }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, FileTransferChunk { target_id: from, file_name, chunk_index, total_chunks, data_base64 }).await;
                }

                Ok(FileTransferComplete { target_id, file_name }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, FileTransferComplete { target_id: from, file_name }).await;
                }

                Ok(FileListRequest { target_id, path }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, FileListRequest { target_id: from, path }).await;
                }

                Ok(FileListResponse { target_id, path, entries }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, FileListResponse { target_id: from, path, entries }).await;
                }

                Ok(FileDownloadRequest { target_id, remote_file_path }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, FileDownloadRequest { target_id: from, remote_file_path }).await;
                }

                Ok(VideoFrame { target_id, width, height, jpeg_base64 }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, VideoFrame { target_id: from, width, height, jpeg_base64 }).await;
                }

                Ok(Control { target_id, event }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, Control { target_id: from, event }).await;
                }

                Ok(Offer { target_id, sdp }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, Offer { target_id: from, sdp }).await;
                }

                Ok(Answer { target_id, sdp }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(&peers, &target_id, Answer { target_id: from, sdp }).await;
                }

                Ok(IceCandidate { target_id, candidate, sdp_mid, sdp_mline_index }) => {
                    let from = current_raw_id.clone().unwrap_or_default();
                    forward_to_target(
                        &peers,
                        &target_id,
                        IceCandidate { target_id: from, candidate, sdp_mid, sdp_mline_index },
                    ).await;
                }

                Ok(msg) => {
                    log::warn!("Mensaje de señalización no manejado: {:?}", msg);
                }

                Err(e) => {
                    log::error!("Error al parsear mensaje JSON: {}. Texto: {}", e, text);
                }
            }
        }
    }

    if let Some(norm) = current_norm_id {
        peers.write().await.remove(&norm);
        println!(">> [{}] Cliente desconectado: ID='{}'", current_brand.to_uppercase(), current_raw_id.as_deref().unwrap_or(&norm));

        if let Some(peer_norm) = sessions.write().await.remove(&norm) {
            sessions.write().await.remove(&peer_norm);
            let raw_from = current_raw_id.unwrap_or_else(|| norm.clone());
            forward_to_target(
                &peers,
                &peer_norm,
                Disconnect {
                    target_id: raw_from,
                    reason: "El equipo remoto ha perdido la conexión de red o se ha cerrado.".to_string(),
                },
            ).await;
        }
    }
}

async fn forward_to_target(peers: &Peers, target_id: &str, msg: SignalMessage) {
    let norm = normalize_id(target_id);
    if let Some(target_peer) = peers.read().await.get(&norm) {
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = target_peer.sender.send(Ok(Message::text(json)));
        }
    }
}
