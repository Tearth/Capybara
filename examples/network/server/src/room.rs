use crate::core::QueuePacket;
use capybara::network::server::client::WebSocketConnectedClientSlim;

pub struct Room {}

impl Room {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, clients: Vec<WebSocketConnectedClientSlim>, packets: Vec<QueuePacket>) {
        //
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
