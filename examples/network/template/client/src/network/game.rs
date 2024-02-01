use capybara::error_continue;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use capybara::network::packet::Packet;
use capybara::rustc_hash::FxHashMap;
use log::info;
use network_template_base::game::GameState;
use network_template_base::packets::*;
use network_template_base::*;
use std::collections::VecDeque;
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

    pub tick: u32,
    pub server_time_offset: u32,
    pub server_time_offset_chunks: Vec<u32>,

    pub server_state: Option<ServerState>,
    pub input_history: VecDeque<InputHistory>,
    pub corrected_nodes: Vec<Vec2>,
}

pub struct ServerState {
    pub timestamp: Instant,
    pub players: FxHashMap<u64, PacketTickData>,
}

pub struct InputHistory {
    pub timestamp: Instant,
    pub heading: f32,
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

            let mut server_state_received = false;

            while let Some(packet) = self.hub_websocket.poll_packet() {
                match packet.get_id() {
                    Some(PACKET_TICK) => {
                        let (header, data) = packet.to_array_with_header::<PacketTickHeader, PacketTickData>().unwrap();
                        let players = data.iter().map(|p| (p.player_id, p.clone())).collect::<FxHashMap<_, _>>();

                        self.server_state = Some(ServerState { timestamp: header.timestamp, players });
                        server_state_received = true;
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
                            self.tick = response.tick;
                        }
                        Err(err) => error_continue!("Failed to parse packet ({})", err),
                    },
                    Some(PACKET_SET_TICK_INTERVAL) => match packet.to_object::<PacketSetTickInterval>() {
                        Ok(response) => {
                            self.tick = response.tick;
                        }
                        Err(err) => error_continue!("Failed to parse packet ({})", err),
                    },
                    _ => {}
                }
            }

            if server_state_received {
                self.process_new_server_state();
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

    pub fn send_new_heading(&mut self, heading: f32) {
        let now = Instant::now();

        self.hub_websocket.send_packet(Packet::from_object(
            PACKET_PLAYER_INPUT,
            &PacketPlayerInput { timestamp: now + Duration::from_millis(self.server_time_offset as u64), heading },
        ));
        self.input_history.push_front(InputHistory { timestamp: now, heading });

        if self.input_history.len() > 100 {
            self.input_history.pop_back();
        }
    }

    pub fn process_new_server_state(&mut self) {
        let now = Instant::now();

        if let Some(server_state) = &self.server_state {
            if let Some(player_state) = server_state.players.get(&self.player_id) {
                let timespan = (now - server_state.timestamp).as_millis();
                let ticks = timespan as u64 / self.tick as u64;
                let tick_remaining = timespan as u64 % self.tick as u64;
                let mut tick_timestamp = server_state.timestamp;

                let mut heading_real = player_state.heading;
                let mut nodes = player_state.nodes.to_vec();

                for _ in 0..ticks {
                    let mut heading_target = None;
                    for input in self.input_history.iter().rev() {
                        if tick_timestamp < input.timestamp {
                            heading_target = Some(input.heading);
                            break;
                        }
                    }

                    if heading_target.is_none() {
                        heading_target = self.input_history.front().map(|p| p.heading);
                    }

                    let heading_target = heading_target.unwrap_or(0.0);
                    let result = game::simulate(GameState { nodes: nodes.clone(), heading_real, heading_target }, self.tick as f32 / 1000.0);

                    heading_real = result.heading_real;
                    nodes = result.nodes;
                    tick_timestamp += Duration::from_millis(self.tick as u64);
                }

                let heading_target = self.input_history.front().map(|p| p.heading).unwrap_or(0.0);
                let result = game::simulate(GameState { nodes: nodes.clone(), heading_real, heading_target }, tick_remaining as f32 / 1000.0);

                self.corrected_nodes = result.nodes;
            }
        }
    }
}
