use crate::error_continue;
use crate::error_return;
use crate::network::packet::Packet;
use futures_channel::mpsc;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use instant::SystemTime;
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
    pub ping: Arc<RwLock<u32>>,

    connected_last_state: bool,
    received_packets: Arc<RwLock<VecDeque<Packet>>>,
    outgoing_packets_tx: Option<UnboundedSender<Packet>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketClient {
    pub fn connect(&mut self, url: &str) {
        info!("Spawning network thread");

        let url = url.to_string();
        let connected = self.connected.clone();
        let ping = self.ping.clone();

        let received_packets = self.received_packets.clone();
        let (outgoing_packets_tx, mut outgoing_packets_rx) = mpsc::unbounded();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();

        self.outgoing_packets_tx = Some(outgoing_packets_tx);
        self.disconnection_tx = Some(disconnection_tx);

        let outgoing_packets_tx = self.outgoing_packets_tx.clone().unwrap();

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
                    Ok((websocket, _)) => websocket,
                    Err(err) => error_return!("Failed to establish connection with the server ({})", err),
                };

                info!("Connection established");

                let (websocket_sink, mut websocket_stream) = websocket.split();
                let (websocket_tx, websocket_rx) = mpsc::unbounded();
                let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

                let process_incoming_messages = async {
                    while let Some(message) = websocket_stream.next().await {
                        if let Ok(Message::Binary(data)) = message {
                            let packet = data.into();

                            match packet {
                                Packet::Ping { timestamp } => {
                                    if let Err(err) = outgoing_packets_tx.unbounded_send(Packet::Pong { timestamp }) {
                                        error_continue!("Failed to send packet ({})", err)
                                    }
                                }
                                Packet::Pong { timestamp } => {
                                    let now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                                        Ok(now) => now.as_millis() as u64,
                                        Err(err) => error_continue!("Failed to obtain current time ({})", err),
                                    };

                                    *ping.write().unwrap() = (now - timestamp) as u32;
                                }
                                _ => {
                                    received_packets.write().unwrap().push_back(packet);
                                }
                            }
                        }
                    }
                };
                let process_outgoing_messages = async {
                    while let Some(packet) = outgoing_packets_rx.next().await {
                        let message = Message::Binary(packet.into());
                        if let Err(err) = websocket_tx.unbounded_send(Ok(message)) {
                            error_continue!("Failed to send packet ({})", err);
                        }
                    }
                };
                let process_disconnection = disconnection_rx.next();

                *connected.write().unwrap() = true;

                tokio::select! {
                    _ = websocket_rx_to_sink => (),
                    _ = process_incoming_messages => (),
                    _ = process_outgoing_messages => (),
                    _ = process_disconnection => ()
                };

                *connected.write().unwrap() = false;
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
        match &self.outgoing_packets_tx {
            Some(outgoing_packets_tx) => {
                if let Err(err) = outgoing_packets_tx.unbounded_send(packet) {
                    error_return!("Failed to send packet ({})", err);
                }
            }
            None => error_return!("Failed to send packet (socket is not connected)"),
        };
    }

    pub fn send_ping(&self) {
        let now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(now) => now.as_millis() as u64,
            Err(err) => error_return!("Failed to obtain current time ({})", err),
        };

        self.send_packet(Packet::Ping { timestamp: now });
    }

    pub fn poll_packet(&mut self) -> Option<Packet> {
        self.received_packets.write().unwrap().pop_front()
    }

    pub fn has_connected(&mut self) -> bool {
        let connected = *self.connected.read().unwrap();
        let has_connected = self.connected_last_state != connected && connected;
        self.connected_last_state = connected;

        has_connected
    }

    pub fn has_disconnected(&mut self) -> bool {
        let connected = *self.connected.read().unwrap();
        let has_disconnected = self.connected_last_state != connected && !connected;
        self.connected_last_state = connected;

        has_disconnected
    }
}
