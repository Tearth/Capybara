use crate::error_return;
use crate::network::frame::Frame;
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
    received_frames: Arc<RwLock<VecDeque<Frame>>>,
    outgoing_frames_tx: Option<UnboundedSender<Frame>>,
    disconnection_tx: Option<UnboundedSender<()>>,
}

impl WebSocketClient {
    pub fn connect(&mut self, url: &str) {
        info!("Spawning network thread");

        let url = url.to_string();
        let connected = self.connected.clone();

        let received_frames = self.received_frames.clone();
        let (outgoing_frames_tx, outgoing_frames_rx) = mpsc::unbounded();
        let (disconnection_tx, mut disconnection_rx) = mpsc::unbounded();

        self.outgoing_frames_tx = Some(outgoing_frames_tx);
        self.disconnection_tx = Some(disconnection_tx);

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
                *connected.write().unwrap() = true;

                let (websocket_sink, websocket_stream) = websocket.split();
                let (websocket_tx, websocket_rx) = mpsc::unbounded();
                let websocket_rx_to_sink = websocket_rx.forward(websocket_sink);

                let process_incoming_messages = websocket_stream.for_each(|message| async {
                    match message {
                        Ok(message) => {
                            let frame = match message {
                                Message::Text(text) => Some(Frame::new_text(text)),
                                Message::Binary(data) => Some(Frame::new_binary(data)),
                                _ => None,
                            };

                            if let Some(frame) = frame {
                                received_frames.write().unwrap().push_back(frame);
                            }
                        }
                        Err(err) => error!("Failed to process received message ({})", err),
                    };
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

                tokio::select! {
                    _ = websocket_rx_to_sink => (),
                    _ = process_incoming_messages => (),
                    _ = process_outgoing_messages => (),
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

    pub fn poll_frame(&mut self) -> Option<Frame> {
        self.received_frames.write().unwrap().pop_front()
    }
}
