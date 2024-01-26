use capybara::instant::Instant;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use capybara::network::packet::Packet;
use network_template_base::packets::*;

pub const SERVER_PING_INTERVAL: i32 = 1000;

#[derive(Default)]
pub struct GameNetworkContext {
    pub hub_endpoint: String,
    pub hub_websocket: WebSocketClient,
    pub player_name: String,
    pub last_ping_timestamp: Option<Instant>,
}

pub struct ServerData {
    pub name: String,
    pub flag: String,
    pub address: String,
    pub websocket: WebSocketClient,
}

impl GameNetworkContext {
    pub fn process(&mut self) {
        if matches!(*self.hub_websocket.status.read().unwrap(), ConnectionStatus::Disconnected | ConnectionStatus::Error) {
            self.hub_websocket.connect(&self.hub_endpoint);
        }

        if *self.hub_websocket.status.read().unwrap() == ConnectionStatus::Connected {
            if self.hub_websocket.has_connected() {
                self.hub_websocket.send_packet(Packet::from_object(PACKET_PLAYER_NAME_REQUEST, &PacketPlayerNameRequest {}));
                self.hub_websocket.send_packet(Packet::from_object(PACKET_SERVER_LIST_REQUEST, &PacketServerListRequest {}));
            }

            while let Some(packet) = self.hub_websocket.poll_packet() {
                match packet.get_id() {
                    _ => {}
                }
            }
        }

        if let Some(last_ping_timestamp) = self.last_ping_timestamp {
            let now = Instant::now();
            if (now - last_ping_timestamp).as_millis() >= SERVER_PING_INTERVAL as u128 {
                if *self.hub_websocket.status.read().unwrap() == ConnectionStatus::Connected {
                    self.hub_websocket.send_ping();
                }

                self.last_ping_timestamp = Some(now);
            }
        } else {
            self.last_ping_timestamp = Some(Instant::now());
        }
    }
}
