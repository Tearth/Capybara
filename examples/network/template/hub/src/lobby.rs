use crate::core::QueuePacket;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClientSlim;
use capybara::rustc_hash::FxHashMap;
use network_template_base::packets::*;

pub struct Lobby {}

impl Lobby {
    pub fn new() -> Self {
        Self {}
    }

    pub fn initialize_client(&mut self, client: WebSocketConnectedClientSlim) {}

    pub fn tick(&mut self, clients: &FxHashMap<u64, WebSocketConnectedClientSlim>, packets: Vec<QueuePacket>) {
        for packet in packets {
            match packet.inner.get_id() {
                Some(PACKET_PLAYER_NAME_REQUEST) => {
                    if let Some(client) = clients.get(&packet.client_id) {
                        client.send_packet(Packet::from_object(
                            PACKET_PLAYER_NAME_RESPONSE,
                            &PacketPlayerNameResponse { name: "Funny Fauna".as_bytes_array() },
                        ));
                    }
                }
                Some(PACKET_SERVER_LIST_REQUEST) => {}
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
