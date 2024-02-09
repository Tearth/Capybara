use crate::config::ConfigLoader;
use crate::core::QueuePacket;
use capybara::error_continue;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::network::packet::Packet;
use capybara::rustc_hash::FxHashMap;
use log::info;
use snake_base::game::{simulation, GameState};
use snake_base::packets::*;
use snake_base::*;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug)]
pub struct Room {
    pub state: VecDeque<RoomState>,
    pub players_added: Vec<u64>,
    pub players_removed: Vec<u64>,
}

#[derive(Debug)]
pub struct RoomState {
    pub timestamp: Instant,
    pub players: FxHashMap<u64, RoomPlayer>,
}

#[derive(Clone, Debug)]
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
        self.players_added.push(client_id);
        info!("Player {} added to the room", client_id);
    }

    pub fn remove_player(&mut self, client_id: u64) {
        self.players_removed.push(client_id);
        info!("Player {} removed from the room", client_id);
    }

    pub fn tick(&mut self, packets: &[QueuePacket], config: &ConfigLoader) -> Vec<QueuePacket> {
        let now = Instant::now();
        let mut outgoing_packets = Vec::new();

        if self.state.is_empty() {
            self.state.push_back(RoomState { timestamp: now, players: FxHashMap::default() })
        }

        let last_state = self.state.front().unwrap();
        let mut players = last_state.players.clone();

        for player in &mut players {
            player.1.input_heading = None;
            player.1.input_timestamp = None;
        }

        self.state.push_front(RoomState { timestamp: now, players });
        if self.state.len() > (config.data.input_max_delay / config.data.worker_tick + 1) as usize {
            self.state.pop_back();
        }

        let state = self.state.front_mut().unwrap();

        // Process players added in-between ticks
        while let Some(player_id) = self.players_added.pop() {
            if state.players.contains_key(&player_id) {
                error_continue!("Player with ID {} already exists in the room", player_id);
            }

            let player = RoomPlayer {
                heading_real: 0.0,
                heading_target: 0.0,
                input_heading: None,
                input_timestamp: None,
                nodes: vec![
                    Vec2::new(100.0 + DISTANCE_BETWEEN_NODES * 4.0, 100.0),
                    Vec2::new(100.0 + DISTANCE_BETWEEN_NODES * 3.0, 100.0),
                    Vec2::new(100.0 + DISTANCE_BETWEEN_NODES * 2.0, 100.0),
                    Vec2::new(100.0 + DISTANCE_BETWEEN_NODES * 1.0, 100.0),
                    Vec2::new(100.0 + DISTANCE_BETWEEN_NODES * 0.0, 100.0),
                ],
            };

            state.players.insert(player_id, player);
        }

        // Process players removed in-between ticks
        while let Some(player_id) = self.players_removed.pop() {
            if !state.players.contains_key(&player_id) {
                error_continue!("Player with ID {} does not exists in the room", player_id);
            }

            state.players.remove(&player_id);
        }

        let mut resimulation = FxHashMap::default();

        for packet in packets.iter() {
            match packet.inner.get_id() {
                Some(PACKET_PLAYER_INPUT) => match packet.inner.to_object::<PacketPlayerInput>() {
                    Ok(mut input) => {
                        let oldest_state = self.state.back().unwrap();

                        // Do not accept input which is beyond the oldest saved state, clamp it to the oldest timestamp
                        if input.timestamp < oldest_state.timestamp {
                            input.timestamp = oldest_state.timestamp;
                        }

                        // Do not accept input with too distant timestamp, it would cause sudden jumps for other players - punish
                        // sender by changing it and forcing misprediction on their side
                        if (now - input.timestamp).as_millis() as u32 > config.data.input_max_delay {
                            input.timestamp = now - Duration::from_millis(config.data.input_max_delay as u64);
                        }

                        for (index, state) in self.state.iter_mut().enumerate() {
                            if input.timestamp >= state.timestamp {
                                if let Some(player) = state.players.get_mut(&packet.client_id) {
                                    player.input_heading = Some(input.heading);
                                    player.input_timestamp = Some(input.timestamp);

                                    if index > 0 {
                                        // Always prioritize the oldest state index for resimulation
                                        if index - 1 > resimulation.get(&packet.client_id).cloned().unwrap_or(0) {
                                            resimulation.insert(packet.client_id, index - 1);
                                        }
                                    }
                                } else {
                                    error_continue!("Cannot apply input, player {} does not exists", packet.client_id);
                                }

                                break;
                            }
                        }
                    }
                    Err(err) => error_continue!("Failed to parse packet ({})", err),
                },
                _ => error_continue!("Invalid frame ID ({:?})", packet.inner.get_id()),
            }
        }

        let mut data = Vec::new();
        let state = self.state.front_mut().unwrap();
        let players = state.players.keys().cloned().collect::<Vec<_>>();

        for player_id in players {
            self.simulate(player_id, resimulation.get(&player_id).copied().unwrap_or(0));
        }

        for (player_id, player) in &self.state.front_mut().unwrap().players {
            data.push(PacketTickData {
                player_id: *player_id,
                heading: player.heading_real,
                nodes: [player.nodes[0], player.nodes[1], player.nodes[2], player.nodes[3], player.nodes[4]],
            });
        }

        for player_id in self.state.front_mut().unwrap().players.keys() {
            outgoing_packets.push(QueuePacket {
                client_id: *player_id,
                timestamp: now,
                inner: Packet::from_array_with_header(PACKET_TICK, &PacketTickHeader { timestamp: now }, &data),
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

                        // Simulate part of the state before input was applied
                        let result = simulation::run(
                            GameState {
                                nodes: previous_state_player.nodes,
                                heading_real: previous_state_player.heading_real,
                                heading_target: previous_state_player.heading_target,
                            },
                            (old_heading_time as f32) / 1000.0,
                        );

                        // Simulate part of the input after input being applied
                        let result = simulation::run(
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
                        let result = simulation::run(
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
