use crate::error_return;
use crate::network::packet::Packet;
use futures_channel::mpsc;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use log::error;
use log::info;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(Default)]
pub struct WebSocketClient {
    pub connected: Arc<RwLock<bool>>,
    received_packets: Arc<RwLock<VecDeque<Packet>>>,
    queue_tx: Option<UnboundedSender<Packet>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketClient {
    pub fn connect(&mut self, url: &str) {
        info!("Spawning network thread");

        let url = url.to_string();
        let connected = self.connected.clone();

        let received_packets = self.received_packets.clone();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();
        let (queue_tx, queue_rx) = mpsc::unbounded();

        self.disconnection_tx = Some(disconnection_tx);
        self.queue_tx = Some(queue_tx);

        thread::spawn(move || {
            info!("Creating network runtime");

            let runtime = match Runtime::new() {
                Ok(runtime) => runtime,
                Err(err) => error_return!("Failed to create network runtime ({})", err),
            };

            runtime.block_on(async move {
                info!("Connecting to {}", url);

                let url = match Url::parse(&url) {
                    Ok(url) => url,
                    Err(err) => error_return!("Failed to parse server URL ({})", err),
                };

                let websocket = match tokio_tungstenite::connect_async(url).await {
                    Ok((ws_stream, _)) => ws_stream,
                    Err(err) => error_return!("Failed to establish connection with the server ({})", err),
                };

                info!("Connection established");
                *connected.write().unwrap() = true;

                let (websocket_sink, websocket_stream) = websocket.split();
                let (websocket_tx, websocket_rx) = mpsc::unbounded();
                let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

                let process_queue_messages = queue_rx.for_each(|packet| async {
                    let message = match packet {
                        Packet::Text { text } => Message::Text(text),
                        Packet::Binary { data } => Message::Binary(data),
                    };

                    if let Err(err) = websocket_tx.unbounded_send(Ok(message)) {
                        error!("Failed to send packet ({})", err);
                    }
                });
                let process_websocket_messages = websocket_stream.for_each(|message| async {
                    match message {
                        Ok(message) => {
                            let packet = match message {
                                Message::Text(text) => Some(Packet::new_text(text)),
                                Message::Binary(data) => Some(Packet::new_binary(data)),
                                _ => None,
                            };

                            if let Some(packet) = packet {
                                received_packets.write().unwrap().push_back(packet);
                            }
                        }
                        Err(err) => error!("Failed to process received message ({})", err),
                    };
                });
                let process_disconnection = disconnection_rx.next();

                tokio::select! {
                    _ = websocket_rx_to_sink => (),
                    _ = process_queue_messages => (),
                    _ = process_websocket_messages => (),
                    _ = process_disconnection => ()
                };

                *connected.write().unwrap() = true;
            });

            info!("Connection closed, network runtime completed");
        });
    }

    pub fn disconnect(&self) {
        match &self.disconnection_tx {
            Some(disconnection_tx) => {
                if let Err(err) = disconnection_tx.unbounded_send(()) {
                    error_return!("Failed to disconnect ({})", err);
                }
            }
            None => error_return!("Failed to disconnect (socket is not connected)"),
        };
    }

    pub fn send_packet(&self, packet: Packet) {
        match &self.queue_tx {
            Some(queue_tx) => {
                if let Err(err) = queue_tx.unbounded_send(packet) {
                    error_return!("Failed to send packet ({})", err);
                }
            }
            None => error_return!("Failed to send packet (socket is not connected)"),
        };
    }

    pub fn poll_packet(&mut self) -> Option<Packet> {
        self.received_packets.write().unwrap().pop_front()
    }
}
