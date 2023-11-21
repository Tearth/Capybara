use crate::error_return;
use crate::network::packet::Packet;
use instant::SystemTime;
use js_sys::Uint8Array;
use log::error;
use log::info;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::BinaryType;
use web_sys::MessageEvent;
use web_sys::WebSocket;

pub struct WebSocketClient {
    pub connected: Arc<RwLock<bool>>,
    pub ping: Arc<RwLock<u32>>,

    websocket: Option<Rc<WebSocket>>,
    received_packets: Arc<RwLock<VecDeque<Packet>>>,

    onopen_callback: Closure<dyn FnMut()>,
    onclose_callback: Closure<dyn FnMut()>,
    onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
    onerror_callback: Closure<dyn FnMut(MessageEvent)>,
}

impl WebSocketClient {
    pub fn new() -> Self {
        Self {
            connected: Default::default(),
            ping: Default::default(),
            websocket: None,
            onopen_callback: Closure::<dyn FnMut()>::new(|| {}),
            onclose_callback: Closure::<dyn FnMut()>::new(|| {}),
            onmessage_callback: Closure::<dyn FnMut(_)>::new(|_| {}),
            onerror_callback: Closure::<dyn FnMut(_)>::new(|_| {}),
            received_packets: Default::default(),
        }
    }

    pub fn connect(&mut self, url: &str) {
        info!("Connecting to {}", url);

        let websocket = match WebSocket::new(url) {
            Ok(websocket) => websocket,
            Err(_) => error_return!("Failed to establish connection with the server"),
        };
        websocket.set_binary_type(BinaryType::Arraybuffer);

        self.websocket = Some(Rc::new(websocket));

        self.init_onopen_callback();
        self.init_onclose_callback();
        self.init_onmessage_callback();
        self.init_onerror_callback();
    }

    fn init_onopen_callback(&mut self) {
        let connected = self.connected.clone();
        self.onopen_callback = Closure::<dyn FnMut()>::new(move || {
            info!("Connection established");
            *connected.write().unwrap() = true;
        });

        match &self.websocket {
            Some(websocket) => {
                let onopen_callback = self.onopen_callback.as_ref().unchecked_ref();
                websocket.set_onopen(Some(onopen_callback));
            }
            None => error_return!("Failed to initialize onopen callback (socket is not connected)"),
        }
    }

    fn init_onclose_callback(&mut self) {
        let connected = self.connected.clone();
        self.onclose_callback = Closure::<dyn FnMut()>::new(move || {
            info!("Connection closed");
            *connected.write().unwrap() = false;
        });

        match &self.websocket {
            Some(websocket) => {
                let onclose_callback = self.onclose_callback.as_ref().unchecked_ref();
                websocket.set_onclose(Some(onclose_callback));
            }
            None => error_return!("Failed to initialize onclose callback (socket is not connected)"),
        }
    }

    fn init_onmessage_callback(&mut self) {
        let ping = self.ping.clone();
        let websocket = self.websocket.clone();
        let received_packets = self.received_packets.clone();

        self.onmessage_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
            if let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = Uint8Array::new(&buffer);
                let length = array.byte_length() as usize;

                let mut data = vec![0; length];
                array.copy_to(&mut data);

                let packet = data.into();
                match packet {
                    Packet::Ping { timestamp } => match &websocket {
                        Some(websocket) => {
                            let packet = Packet::Pong { timestamp };
                            let data: Vec<u8> = packet.into();

                            websocket.send_with_u8_array(&data).unwrap();
                        }
                        None => error_return!("Failed to send packet (socket is not connected)"),
                    },
                    Packet::Pong { timestamp } => {
                        let now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                            Ok(now) => now.as_millis(),
                            Err(_) => error_return!("Failed to obtain current time"),
                        };

                        *ping.write().unwrap() = (now - timestamp) as u32;
                    }
                    _ => {
                        received_packets.write().unwrap().push_back(packet);
                    }
                }
            }
        });

        match &self.websocket {
            Some(websocket) => {
                let onmessage_callback = self.onmessage_callback.as_ref().unchecked_ref();
                websocket.set_onmessage(Some(onmessage_callback));
            }
            None => error_return!("Failed to initialize onmessage callback (socket is not connected)"),
        }
    }

    fn init_onerror_callback(&mut self) {
        self.onerror_callback = Closure::<dyn FnMut(_)>::new(move |_| {
            error!("Connection error");
        });

        match &self.websocket {
            Some(websocket) => {
                let onerror_callback = self.onerror_callback.as_ref().unchecked_ref();
                websocket.set_onerror(Some(onerror_callback));
            }
            None => error_return!("Failed to initialize onerror callback (socket is not connected)"),
        }
    }

    pub fn disconnect(&self) {
        match &self.websocket {
            Some(websocket) => {
                if websocket.close().is_err() {
                    error_return!("Failed to disconnect");
                }
            }
            None => error_return!("Failed to disconnect (socket is not connected)"),
        }
    }

    pub fn send_packet(&self, packet: Packet) {
        if let Some(websocket) = &self.websocket {
            let data: Vec<u8> = packet.into();
            if websocket.send_with_u8_array(&data).is_err() {
                error_return!("Failed to send packet");
            }
        } else {
            error_return!("Failed to send packet (socket is not connected)");
        }
    }

    pub fn poll_packet(&mut self) -> Option<Packet> {
        self.received_packets.write().unwrap().pop_front()
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        if let Some(websocket) = &self.websocket {
            websocket.set_onopen(None);
            websocket.set_onclose(None);
            websocket.set_onmessage(None);
            websocket.set_onerror(None);
        }
    }
}
