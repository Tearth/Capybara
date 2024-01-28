use capybara::error_continue;
use capybara::instant::Instant;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use capybara::network::packet::Packet;
use log::info;
use network_template_base::packets::*;
use std::time::Duration;

pub const SERVER_PING_INTERVAL: i32 = 1000;
pub const SERVER_TIME_REQUEST_TRIES: usize = 5;

#[derive(Default)]
pub struct GameNetworkContext {
    pub hub_name: String,
    pub hub_endpoint: String,
    pub hub_websocket: WebSocketClient,
    pub player_id: u64,
    pub player_name: String,
    pub last_ping_timestamp: Option<Instant>,
    pub last_server_request_timestamp: Option<Instant>,

    pub server_time_offset: u32,
    pub server_time_offset_chunks: Vec<u32>,

    pub state: Vec<PacketTickData>,
}

pub struct GameState {
    pub state: Vec<PacketTickData>,
}

impl GameNetworkContext {
    pub fn process(&mut self) {
        let now = Instant::now();

        if matches!(*self.hub_websocket.status.read().unwrap(), ConnectionStatus::Disconnected | ConnectionStatus::Error) {
            info!("Server {} is disconnected, restarting connection", self.hub_name);
            self.hub_websocket.connect(&self.hub_endpoint);
        }

        if *self.hub_websocket.status.read().unwrap() == ConnectionStatus::Connected {
            if self.hub_websocket.has_connected() {
                self.hub_websocket.send_packet(Packet::from_object(PACKET_SERVER_TIME_REQUEST, &PacketServerTimeRequest {}));

                self.last_server_request_timestamp = Some(now);
            }

            while let Some(packet) = self.hub_websocket.poll_packet() {
                match packet.get_id() {
                    Some(PACKET_TICK) => {
                        let (header, data) = packet.to_array_with_header::<PacketTickHeader, PacketTickData>().unwrap();
                        self.state = data;
                    }
                    Some(PACKET_SERVER_TIME_RESPONSE) => match packet.to_object::<PacketServerTimeResponse>() {
                        Ok(response) => {
                            let travel_time = (now - self.last_server_request_timestamp.unwrap()).as_millis();
                            let server_time = response.time + Duration::from_millis((travel_time / 2) as u64);
                            let offset = (server_time - now).as_millis();

                            info!("Received server time offset: {} ms", offset);
                            self.server_time_offset_chunks.push(offset as u32);

                            if self.server_time_offset_chunks.len() < SERVER_TIME_REQUEST_TRIES {
                                self.hub_websocket.send_packet(Packet::from_object(PACKET_SERVER_TIME_REQUEST, &PacketServerTimeRequest {}));
                                self.last_server_request_timestamp = Some(now);
                            } else {
                                self.server_time_offset_chunks.sort_unstable();
                                self.server_time_offset = self.server_time_offset_chunks[SERVER_TIME_REQUEST_TRIES / 2];
                                self.last_server_request_timestamp = None;

                                info!("Final server time offset: {} ms", offset);
                                info!("Joining game room");

                                self.hub_websocket.send_packet(Packet::from_object(PACKET_JOIN_ROOM_REQUEST, &PacketJoinRoomRequest {}));
                            }
                        }
                        Err(err) => error_continue!("Failed to parse packet ({})", err),
                    },
                    Some(PACKET_JOIN_ROOM_RESPONSE) => match packet.to_object::<PacketJoinRoomResponse>() {
                        Ok(response) => {
                            self.player_id = response.player_id;
                        }
                        Err(err) => error_continue!("Failed to parse packet ({})", err),
                    },
                    _ => {}
                }
            }
        }

        if let Some(last_ping_timestamp) = self.last_ping_timestamp {
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
