use crate::core::QueuePacket;
use capybara::error_continue;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::network::packet::Packet;
use capybara::rustc_hash::FxHashMap;
use log::info;
use network_template_base::game::GameState;
use network_template_base::packets::*;
use network_template_base::*;
use std::collections::VecDeque;
use std::f32::consts;
use std::time::Duration;

pub struct Room {
    pub state: VecDeque<RoomState>,
    pub players_added: Vec<u64>,
    pub players_removed: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct RoomState {
    pub timestamp: Instant,
    pub players: FxHashMap<u64, RoomPlayer>,
}

#[derive(Debug, Clone)]
pub struct RoomPlayer {
    pub heading_real: f32,
    pub heading_target: f32,
    pub input_heading: Option<f32>,
    pub input_timestamp: Option<Instant>,
    pub nodes: Vec<Vec2>,
}

impl Room {
    pub fn new() -> Self {
        Self { state: VecDeque::new(), players_added: Vec::new(), players_removed: Vec::new() }
    }

    pub fn add_player(&mut self, client_id: u64) {
        info!("Player {} added to the room", client_id);
        self.players_added.push(client_id);
    }

    pub fn remove_player(&mut self, client_id: u64) {
        info!("Player {} removed from the room", client_id);
        self.players_removed.push(client_id);
    }

    pub fn tick(&mut self, packets: &[QueuePacket]) -> Vec<QueuePacket> {
        let now = Instant::now();
        let mut outgoing_packets = Vec::new();

        if self.state.is_empty() {
            self.state.push_back(RoomState { timestamp: now - Duration::from_millis(TICK), players: FxHashMap::default() })
        }

        let last_state = self.state.front().unwrap();
        let mut players = last_state.players.clone();

        for player in &mut players {
            player.1.input_heading = None;
            player.1.input_timestamp = None;
        }
        self.state.push_front(RoomState { timestamp: now, players });

        if self.state.len() > PRESERVED_STATES_COUNT {
            self.state.pop_back();
        }

        let state = self.state.front_mut().unwrap();

        for client_id in &self.players_added {
            if state.players.contains_key(client_id) {
                error_continue!("Player with ID {} already exists in the room", client_id);
            }

            state.players.insert(
                *client_id,
                RoomPlayer {
                    heading_real: 0.0,
                    heading_target: 0.0,
                    input_heading: None,
                    input_timestamp: None,
                    nodes: vec![
                        Vec2::new(220.0, 100.0),
                        Vec2::new(190.0, 100.0),
                        Vec2::new(160.0, 100.0),
                        Vec2::new(130.0, 100.0),
                        Vec2::new(100.0, 100.0),
                    ],
                },
            );
        }

        for client_id in &self.players_removed {
            if !state.players.contains_key(client_id) {
                error_continue!("Player with ID {} does not exists in the room", client_id);
            }

            state.players.remove(client_id);
        }

        self.players_added.clear();
        self.players_removed.clear();

        let mut players_to_resimulate = FxHashMap::default();

        for packet in packets.iter() {
            match packet.inner.get_id() {
                Some(PACKET_PLAYER_INPUT) => match packet.inner.to_object::<PacketPlayerInput>() {
                    Ok(input) => {
                        let oldest_state = self.state.back().unwrap();
                        if input.timestamp < oldest_state.timestamp {
                            // Input is too old, discard it
                            continue;
                        }

                        for (index, state) in self.state.iter_mut().enumerate() {
                            let offset = Duration::from_millis(0);
                            if input.timestamp + offset >= state.timestamp {
                                if let Some(player) = state.players.get_mut(&packet.client_id) {
                                    player.input_heading = Some(input.heading);
                                    player.input_timestamp = Some(input.timestamp + offset);

                                    if index > 0 && !players_to_resimulate.contains_key(&packet.client_id) {
                                        players_to_resimulate.insert(packet.client_id, 19 - 1);
                                    }
                                } else {
                                    error_continue!("Player not found");
                                }

                                break;
                            }
                        }
                    }
                    Err(err) => error_continue!("Failed to parse packet ({})", err),
                },
                _ => {}
            }
        }

        let state = self.state.front_mut().unwrap();
        let players_to_simulate = state.players.keys().cloned().collect::<Vec<_>>();

        for client_id in players_to_simulate {
            let from_state_index = match players_to_resimulate.get(&client_id) {
                Some(state_index) => *state_index,
                None => 0,
            };

            self.simulate(client_id, from_state_index);
        }

        let header = PacketTickHeader { timestamp: now };
        let mut data = Vec::new();

        for (client_id, player) in &self.state.front_mut().unwrap().players {
            data.push(PacketTickData {
                player_id: *client_id,
                nodes: [player.nodes[0], player.nodes[1], player.nodes[2], player.nodes[3], player.nodes[4]],
            });
        }

        for client_id in self.state.front_mut().unwrap().players.keys() {
            outgoing_packets.push(QueuePacket {
                client_id: *client_id,
                timestamp: now,
                inner: Packet::from_array_with_header(PACKET_TICK, &header, &data),
            });
        }

        outgoing_packets
    }

    fn simulate(&mut self, player_id: u64, from_state_index: usize) {
        let mut previous_state_index = from_state_index + 1;
        let mut current_state_index = from_state_index;

        loop {
            let previous_state_timestamp = self.state[previous_state_index].timestamp;
            let current_state_timestamp = self.state[current_state_index].timestamp;
            let delta = (current_state_timestamp - previous_state_timestamp).as_millis();

            if let Some(previous_state_player) = self.state[previous_state_index].players.get(&player_id).cloned() {
                if let Some(current_state_player) = self.state[current_state_index].players.get_mut(&player_id) {
                    if let Some(previous_heading_input) = previous_state_player.input_heading {
                        current_state_player.heading_target = previous_heading_input;
                    } else {
                        current_state_player.heading_target = previous_state_player.heading_target;
                    }

                    if let Some(previous_input_timestamp) = previous_state_player.input_timestamp {
                        let old_heading_time = (previous_input_timestamp - previous_state_timestamp).as_millis();
                        let new_heading_time = (current_state_timestamp - previous_input_timestamp).as_millis();

                        let result = game::simulate(
                            GameState {
                                nodes: previous_state_player.nodes,
                                heading_real: previous_state_player.heading_real,
                                heading_target: previous_state_player.heading_target,
                            },
                            (old_heading_time as f32) / 1000.0,
                        );

                        let result = game::simulate(
                            GameState {
                                nodes: result.nodes,
                                heading_real: result.heading_real,
                                heading_target: previous_state_player.input_heading.unwrap(),
                            },
                            (new_heading_time as f32) / 1000.0,
                        );

                        current_state_player.heading_real = result.heading_real;
                        current_state_player.nodes = result.nodes;
                    } else {
                        let result = game::simulate(
                            GameState {
                                nodes: previous_state_player.nodes,
                                heading_real: previous_state_player.heading_real,
                                heading_target: previous_state_player.heading_target,
                            },
                            (delta as f32) / 1000.0,
                        );

                        current_state_player.heading_real = result.heading_real;
                        current_state_player.nodes = result.nodes;
                    }
                }
            }

            if current_state_index == 0 {
                break;
            }

            previous_state_index -= 1;
            current_state_index -= 1;
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
