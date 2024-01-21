use crate::core::QueuePacket;
use capybara::network::server::client::WebSocketConnectedClientSlim;

pub struct Lobby {}

impl Lobby {
    pub fn new() -> Self {
        Self {}
    }

    pub fn initialize_client(&mut self, client: WebSocketConnectedClientSlim) {}

    pub fn tick(&mut self, clients: Vec<WebSocketConnectedClientSlim>, packets: Vec<QueuePacket>) {
        for packet in packets {
            match packet.inner.get_id() {
                _ => {}
            }
        }
    }
}

impl Default for Lobby {
    fn default() -> Self {
        Self::new()
    }
}
