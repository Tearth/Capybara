use std::sync::{Arc, RwLock};
use std::time::UNIX_EPOCH;

use crate::error_return;
use crate::network::frame::Frame;
use crate::network::packet::Packet;
use futures_channel::mpsc;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use instant::{Duration, Instant, SystemTime};
use log::error;
use tokio::net::TcpStream;
use tokio::select;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketConnectedClient {
    pub id: u64,
    pub ping: Arc<RwLock<u32>>,

    websocket: Option<WebSocketStream<TcpStream>>,
    outgoing_frames_tx: Option<UnboundedSender<Packet>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketConnectedClient {
    pub fn new(websocket: WebSocketStream<TcpStream>) -> Self {
        Self { id: fastrand::u64(..), ping: Default::default(), websocket: Some(websocket), outgoing_frames_tx: None, disconnection_tx: None }
    }

    pub fn run(&mut self, incoming_frames_tx: UnboundedSender<(u64, Packet)>) {
        let id = self.id;
        let websocket = self.websocket.take().unwrap();

        let (websocket_sink, websocket_stream) = websocket.split();
        let (websocket_tx, websocket_rx) = mpsc::unbounded();
        let (outgoing_frames_tx, outgoing_frames_rx) = mpsc::unbounded();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();
        let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

        let ping = self.ping.clone();
        let outgoing_frames_tx_clone = outgoing_frames_tx.clone();

        self.outgoing_frames_tx = Some(outgoing_frames_tx);
        self.disconnection_tx = Some(disconnection_tx);

        tokio::spawn(async move {
            let process_incoming_messages = websocket_stream.for_each(|message| async {
                let frame = match message.unwrap() {
                    Message::Text(text) => Some(Frame::new_text(text)),
                    Message::Binary(data) => Some(Frame::new_binary(data)),
                    _ => None,
                };

                if let Some(frame) = frame {
                    let packet = frame.into();

                    match packet {
                        Packet::Ping { timestamp } => {
                            if let Err(err) = outgoing_frames_tx_clone.unbounded_send(Packet::Pong { timestamp }) {
                                error_return!("Failed to send frame ({})", err)
                            }
                        }
                        Packet::Pong { timestamp } => {
                            *ping.write().unwrap() =
                                (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() - timestamp) as u32;
                        }
                        _ => {
                            if let Err(err) = incoming_frames_tx.unbounded_send((id, packet)) {
                                error!("Failed to process frame ({})", err);
                            }
                        }
                    }
                }
            });
            let process_outgoing_messages = outgoing_frames_rx.for_each(|packet| async {
                let message = match packet.into() {
                    Frame::Text { text } => Some(Message::Text(text)),
                    Frame::Binary { data } => Some(Message::Binary(data)),
                    Frame::Unknown => None,
                };

                match message {
                    Some(message) => {
                        if let Err(err) = websocket_tx.unbounded_send(Ok(message)) {
                            error!("Failed to send frame ({})", err);
                        }
                    }
                    None => error!("Failed to parse message"),
                };
            });
            let process_disconnection = disconnection_rx.next();

            select! {
                _ = process_incoming_messages => {}
                _ = process_outgoing_messages => {}
                _ = websocket_rx_to_sink => {}
                _ = process_disconnection => {}
            }
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

    pub fn send_frame(&self, packet: Packet) {
        match &self.outgoing_frames_tx {
            Some(outgoing_frames_tx) => {
                if let Err(err) = outgoing_frames_tx.unbounded_send(packet) {
                    error_return!("Failed to send frame ({})", err);
                }
            }
            None => error_return!("Failed to send frame (socket is not connected)"),
        };
    }

    pub fn send_ping(&self) {
        self.send_frame(Packet::Ping { timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() });
    }
}
