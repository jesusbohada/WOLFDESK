use futures::{SinkExt, StreamExt};
use proto::{Identity, SignalMessage};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

pub struct SignalingClient {
    pub identity: Identity,
    server_url: String,
}

impl SignalingClient {
    pub fn new(server_url: &str, identity: Identity) -> Self {
        Self {
            identity,
            server_url: server_url.to_string(),
        }
    }

    /// Inicia el bucle de conexión con el servidor de señalización con reconexión automática permanente
    pub async fn start(
        &self,
        mut outbound_rx: mpsc::UnboundedReceiver<SignalMessage>,
        inbound_tx: mpsc::UnboundedSender<SignalMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&self.server_url)?;

        loop {
            log::info!("Conectando al servidor de señalización: {}", url);

            let (ws_stream, _) = match connect_async(url.clone()).await {
                Ok(stream) => stream,
                Err(e) => {
                    log::warn!("No se pudo conectar al servidor de señalización: {}. Reintentando en 3s...", e);
                    let _ = inbound_tx.send(SignalMessage::Error {
                        message: "Sin conexión con el servidor de señalización. Reconectando...".to_string(),
                    });
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    continue;
                }
            };

            let (mut write, mut read) = ws_stream.split();

            // 1. Envía mensaje de registro inicial con el ID de 9 dígitos y la clave pública
            let reg_msg = SignalMessage::Register {
                client_id: self.identity.numeric_id.clone(),
                public_key: self.identity.public_key_hex(),
            };
            if let Ok(reg_json) = serde_json::to_string(&reg_msg) {
                if write.send(Message::Text(reg_json)).await.is_err() {
                    log::warn!("Error al enviar registro de cliente. Reintentando...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            }
            log::info!("Registrado en el servidor con ID: {}", self.identity.numeric_id);

            // Bucle principal concurrente de envío y recepción
            loop {
                tokio::select! {
                    msg_opt = outbound_rx.recv() => {
                        match msg_opt {
                            Some(msg) => {
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    if write.send(Message::Text(json)).await.is_err() {
                                        log::warn!("Error al enviar mensaje por WebSocket. Reconectando...");
                                        break;
                                    }
                                }
                            }
                            None => {
                                log::warn!("Canal de salida cerrado.");
                                return Ok(());
                            }
                        }
                    }
                    read_opt = read.next() => {
                        match read_opt {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(signal_msg) = serde_json::from_str::<SignalMessage>(&text) {
                                    let _ = inbound_tx.send(signal_msg);
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                log::warn!("Conexión cerrada por el servidor de señalización. Reconectando...");
                                break;
                            }
                            Some(Err(e)) => {
                                log::warn!("Error en la conexión WebSocket: {}. Reconectando...", e);
                                break;
                            }
                            None => {
                                log::warn!("Flujo WebSocket terminado. Reconectando...");
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }

            let _ = inbound_tx.send(SignalMessage::Error {
                message: "Conexión perdida con el servidor. Reconectando automáticamente...".to_string(),
            });
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}
