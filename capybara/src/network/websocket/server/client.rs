use crate::error_return;
use crate::network::packet::Packet;
use futures_channel::mpsc;
use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use log::error;
use tokio::net::TcpStream;
use tokio::select;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketConnectedClient {
    websocket: Option<WebSocketStream<TcpStream>>,
    outgoing_packets_tx: Option<UnboundedSender<Packet>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketConnectedClient {
    pub fn new(websocket: WebSocketStream<TcpStream>) -> Self {
        Self { websocket: Some(websocket), outgoing_packets_tx: None, disconnection_tx: None }
    }

    pub fn run(&mut self, incoming_packets_tx: UnboundedSender<Packet>) {
        let websocket = self.websocket.take().unwrap();

        let (websocket_sink, websocket_stream) = websocket.split();
        let (websocket_tx, websocket_rx) = mpsc::unbounded();
        let (outgoing_packets_tx, outgoing_packets_rx) = mpsc::unbounded();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();
        let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

        self.outgoing_packets_tx = Some(outgoing_packets_tx);
        self.disconnection_tx = Some(disconnection_tx);

        tokio::spawn(async move {
            let process_incoming_messages = websocket_stream.for_each(|message| async {
                let packet = match message.unwrap() {
                    Message::Text(text) => Some(Packet::new_text(text)),
                    Message::Binary(data) => Some(Packet::new_binary(data)),
                    _ => None,
                };

                if let Some(packet) = packet {
                    if let Err(err) = incoming_packets_tx.unbounded_send(packet) {
                        error!("Failed to process packet ({})", err);
                    }
                }
            });
            let process_outgoing_messages = outgoing_packets_rx.for_each(|packet| async {
                let message = match packet {
                    Packet::Text { text } => Message::Text(text),
                    Packet::Binary { data } => Message::Binary(data),
                };

                if let Err(err) = websocket_tx.unbounded_send(Ok(message)) {
                    error!("Failed to send packet ({})", err);
                }
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

    pub fn send_packet(&self, packet: Packet) {
        match &self.outgoing_packets_tx {
            Some(queue_tx) => {
                if let Err(err) = queue_tx.unbounded_send(packet) {
                    error_return!("Failed to send packet ({})", err);
                }
            }
            None => error_return!("Failed to send packet (socket is not connected)"),
        };
    }
}
