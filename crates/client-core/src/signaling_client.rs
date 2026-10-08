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

    /// Inicia el bucle de conexión con el servidor de señalización
    pub async fn start(
        &self,
        mut outbound_rx: mpsc::UnboundedReceiver<SignalMessage>,
        inbound_tx: mpsc::UnboundedSender<SignalMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&self.server_url)?;
        log::info!("Conectando al servidor de señalización: {}", url);

        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        // 1. Envía mensaje de registro inicial con el ID de 9 dígitos y la clave pública
        let reg_msg = SignalMessage::Register {
            client_id: self.identity.numeric_id.clone(),
            public_key: self.identity.public_key_hex(),
        };
        let reg_json = serde_json::to_string(&reg_msg)?;
        write.send(Message::Text(reg_json)).await?;
        log::info!("Registrado en el servidor con ID: {}", self.identity.numeric_id);

        // Tarea para enviar mensajes que salgan de la aplicación hacia el servidor
        let send_task = tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if write.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Bucle para recibir mensajes entrantes del servidor
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(signal_msg) = serde_json::from_str::<SignalMessage>(&text) {
                        let _ = inbound_tx.send(signal_msg);
                    }
                }
                Ok(Message::Close(_)) => {
                    log::warn!("Conexión cerrada por el servidor de señalización.");
                    break;
                }
                Err(e) => {
                    log::error!("Error en la conexión WebSocket: {}", e);
                    break;
                }
                _ => {}
            }
        }

        send_task.abort();
        Ok(())
    }
}
