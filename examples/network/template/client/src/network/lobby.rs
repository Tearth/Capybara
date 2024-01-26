use capybara::error_continue;
use capybara::instant::Instant;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use capybara::network::packet::Packet;
use log::info;
use network_template_base::packets::*;

pub const HUB_ENDPOINT: &str = "ws://localhost:10000";
pub const SERVER_PING_INTERVAL: i32 = 1000;

#[derive(Default)]
pub struct LobbyNetworkContext {
    pub hub_websocket: WebSocketClient,
    pub player_name: String,
    pub servers: Vec<ServerData>,
    pub last_ping_timestamp: Option<Instant>,
}

pub struct ServerData {
    pub name: String,
    pub flag: String,
    pub address: String,
    pub websocket: WebSocketClient,
}

impl LobbyNetworkContext {
    pub fn process(&mut self) {
        if matches!(*self.hub_websocket.status.read().unwrap(), ConnectionStatus::Disconnected | ConnectionStatus::Error) {
            self.hub_websocket.connect(HUB_ENDPOINT);
        }

        if *self.hub_websocket.status.read().unwrap() == ConnectionStatus::Connected {
            if self.hub_websocket.has_connected() {
                self.hub_websocket.send_packet(Packet::from_object(PACKET_PLAYER_NAME_REQUEST, &PacketPlayerNameRequest {}));
                self.hub_websocket.send_packet(Packet::from_object(PACKET_SERVER_LIST_REQUEST, &PacketServerListRequest {}));
            }

            while let Some(packet) = self.hub_websocket.poll_packet() {
                match packet.get_id() {
                    Some(PACKET_PLAYER_NAME_RESPONSE) => {
                        let packet = match packet.to_object::<PacketPlayerNameResponse>() {
                            Ok(packet) => packet,
                            Err(err) => error_continue!("Invalid packet ({})", err),
                        };

                        self.player_name = String::from_utf8_lossy(&packet.name).trim_end_matches('\0').to_string();
                        info!("Received new player name: {}", self.player_name);
                    }
                    Some(PACKET_SERVER_LIST_RESPONSE) => {
                        let array = match packet.to_array::<PacketServerListResponse>() {
                            Ok(packet) => packet,
                            Err(err) => error_continue!("Invalid packet ({})", err),
                        };

                        info!("Received a list of servers");

                        for server in array {
                            let data = ServerData {
                                name: String::from_utf8_lossy(&server.name).trim_end_matches('\0').to_string(),
                                flag: String::from_utf8_lossy(&server.flag).trim_end_matches('\0').to_string(),
                                address: String::from_utf8_lossy(&server.address).trim_end_matches('\0').to_string(),
                                websocket: Default::default(),
                            };

                            info!("{} ({}) - {}", data.name, data.flag, data.address);
                            self.servers.push(data);
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(last_ping_timestamp) = self.last_ping_timestamp {
            let now = Instant::now();
            if (now - last_ping_timestamp).as_millis() >= SERVER_PING_INTERVAL as u128 {
                for server in &self.servers {
                    if *server.websocket.status.read().unwrap() == ConnectionStatus::Connected {
                        server.websocket.send_ping();
                    }
                }

                self.last_ping_timestamp = Some(now);
            }
        } else {
            self.last_ping_timestamp = Some(Instant::now());
        }
    }
}
