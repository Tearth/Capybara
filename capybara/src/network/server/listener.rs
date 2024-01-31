use super::client::WebSocketConnectedClient;
use crate::error_continue;
use crate::error_return;
use futures_channel::mpsc;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use log::error;
use log::info;
use tokio::net::TcpListener;
use tokio::select;

#[derive(Debug)]
pub struct WebSocketListener {
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketListener {
    pub fn new() -> Self {
        Self { disconnection_tx: None }
    }

    pub async fn listen(&mut self, address: &str, client_event: UnboundedSender<WebSocketConnectedClient>) {
        info!("Creating a TCP listener at {}", address);
        let tcp_listener = match TcpListener::bind(&address).await {
            Ok(tcp_listener) => tcp_listener,
            Err(err) => error_return!("Failed to create TCP listener ({})", err),
        };

        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();
        self.disconnection_tx = Some(disconnection_tx);

        let listen = tokio::spawn(async move {
            while let Ok((stream, address)) = tcp_listener.accept().await {
                let id = fastrand::u64(..);
                info!("New client {} connected ({})", id, address);

                let websocket = match tokio_tungstenite::accept_async(stream).await {
                    Ok(websocket) => websocket,
                    Err(err) => error_continue!("Failed to accept WebSocket connection ({})", err),
                };
                info!("WebSocket connection for client {} established", id);

                if let Err(err) = client_event.unbounded_send(WebSocketConnectedClient::new(id, websocket, address)) {
                    error!("Failed to send client event ({})", err);
                }
            }
        });
        let process_disconnection = disconnection_rx.next();

        select! {
            _ = listen => {},
            _ = process_disconnection => {}
        }
    }

    pub fn close(&self) {
        match &self.disconnection_tx {
            Some(disconnection_tx) => {
                if let Err(err) = disconnection_tx.unbounded_send(()) {
                    error_return!("Failed to close listener ({})", err);
                }
            }
            None => error_return!("Failed to close listener (socket is not connected)"),
        };
    }
}

impl Default for WebSocketListener {
    fn default() -> Self {
        Self::new()
    }
}
