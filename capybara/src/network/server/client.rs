use crate::error_continue;
use crate::error_return;
use crate::network::packet::Packet;
use futures_channel::mpsc;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use instant::SystemTime;
use log::error;
use log::info;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::net::TcpStream;
use tokio::select;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketConnectedClient {
    pub id: u64,
    pub ping: Arc<RwLock<u32>>,

    websocket: Option<WebSocketStream<TcpStream>>,
    outgoing_packets_tx: Option<UnboundedSender<Packet>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

pub struct WebSocketConnectedClientSlim {
    pub id: u64,

    outgoing_packets_tx: Option<UnboundedSender<Packet>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketConnectedClient {
    pub fn new(websocket: WebSocketStream<TcpStream>) -> Self {
        Self { id: fastrand::u64(..), ping: Default::default(), websocket: Some(websocket), outgoing_packets_tx: None, disconnection_tx: None }
    }

    pub fn run(&mut self, packet_event: UnboundedSender<(u64, Packet)>, disconnection_event: UnboundedSender<u64>) {
        let id = self.id;
        let websocket = self.websocket.take().unwrap();

        info!("Client {} initialized", id);

        let (websocket_sink, mut websocket_stream) = websocket.split();
        let (websocket_tx, websocket_rx) = mpsc::unbounded();
        let (outgoing_packets_tx, mut outgoing_packets_rx) = mpsc::unbounded();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();
        let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

        let ping = self.ping.clone();
        let outgoing_packets_tx_clone = outgoing_packets_tx.clone();

        self.outgoing_packets_tx = Some(outgoing_packets_tx);
        self.disconnection_tx = Some(disconnection_tx);

        tokio::spawn(async move {
            let process_incoming_messages = async {
                while let Some(message) = websocket_stream.next().await {
                    match message {
                        Ok(Message::Binary(data)) => {
                            let packet = data.into();

                            match packet {
                                Packet::Ping { timestamp } => {
                                    if let Err(err) = outgoing_packets_tx_clone.unbounded_send(Packet::Pong { timestamp }) {
                                        error_return!("Failed to send packet ({})", err)
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
                                    if let Err(err) = packet_event.unbounded_send((id, packet)) {
                                        error!("Failed to send packet event ({})", err);
                                    }
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            break;
                        }
                        Err(err) => {
                            error!("Failed to process WebSocket message ({})", err);
                        }
                        _ => {}
                    }
                }
            };
            let process_outgoing_messages = async {
                while let Some(packet) = outgoing_packets_rx.next().await {
                    let message = Message::Binary(packet.into());
                    if let Err(err) = websocket_tx.unbounded_send(Ok(message)) {
                        error!("Failed to send packet ({})", err);
                    }
                }
            };
            let process_disconnection = disconnection_rx.next();

            select! {
                _ = process_incoming_messages => {}
                _ = process_outgoing_messages => {}
                _ = websocket_rx_to_sink => {}
                _ = process_disconnection => {}
            }

            info!("Client {} disconnected", id);

            if let Err(err) = disconnection_event.unbounded_send(id) {
                error_return!("Failed to send disconnection event ({})", err)
            }
        });
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

    pub fn to_slim(&self) -> WebSocketConnectedClientSlim {
        WebSocketConnectedClientSlim {
            id: self.id,
            outgoing_packets_tx: self.outgoing_packets_tx.clone(),
            disconnection_tx: self.disconnection_tx.clone(),
        }
    }
}

impl WebSocketConnectedClientSlim {
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
}
