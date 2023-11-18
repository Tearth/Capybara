use crate::error_return;
use crate::network::frame::Frame;
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
    outgoing_frames_tx: Option<UnboundedSender<Frame>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketConnectedClient {
    pub fn new(websocket: WebSocketStream<TcpStream>) -> Self {
        Self { websocket: Some(websocket), outgoing_frames_tx: None, disconnection_tx: None }
    }

    pub fn run(&mut self, incoming_frames_tx: UnboundedSender<Frame>) {
        let websocket = self.websocket.take().unwrap();

        let (websocket_sink, websocket_stream) = websocket.split();
        let (websocket_tx, websocket_rx) = mpsc::unbounded();
        let (outgoing_frames_tx, outgoing_frames_rx) = mpsc::unbounded();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();
        let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

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
                    if let Err(err) = incoming_frames_tx.unbounded_send(frame) {
                        error!("Failed to process frame ({})", err);
                    }
                }
            });
            let process_outgoing_messages = outgoing_frames_rx.for_each(|frame| async {
                let message = match frame {
                    Frame::Text { text } => Message::Text(text),
                    Frame::Binary { data } => Message::Binary(data),
                };

                if let Err(err) = websocket_tx.unbounded_send(Ok(message)) {
                    error!("Failed to send frame ({})", err);
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

    pub fn send_frame(&self, frame: Frame) {
        match &self.outgoing_frames_tx {
            Some(queue_tx) => {
                if let Err(err) = queue_tx.unbounded_send(frame) {
                    error_return!("Failed to send frame ({})", err);
                }
            }
            None => error_return!("Failed to send frame (socket is not connected)"),
        };
    }
}
