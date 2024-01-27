use crate::core::QueuePacket;
use capybara::error_continue;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::network::packet::Packet;
use capybara::rustc_hash::FxHashMap;
use log::info;
use network_template_base::packets::*;
use network_template_base::*;
use std::collections::VecDeque;
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

    pub fn tick(&mut self, packets: &Vec<QueuePacket>) -> Vec<QueuePacket> {
        let mut outgoing_packets = Vec::new();
        let now = Instant::now();

        if self.state.is_empty() {
            self.state.push_back(RoomState { timestamp: now - Duration::from_millis(TICK), players: FxHashMap::default() })
        }

        let last_state = self.state.front().unwrap();
        self.state.push_front(RoomState { timestamp: now, players: last_state.players.clone() });
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

        for packet in packets.iter() {
            match packet.inner.get_id() {
                _ => {}
            }
        }

        let players_to_simulate = state.players.keys().cloned().collect::<Vec<_>>();
        for client_id in players_to_simulate {
            self.simulate(client_id, 0);
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
            let previous_state = &self.state[previous_state_index];
            let current_state = &self.state[current_state_index];

            // let previous_state_player = previous_state.players.get(&player_id);
            // let current_state_player = current_state.players.get(&player_id);

            let current_state = &mut self.state[current_state_index];
            let current_state_player = current_state.players.get_mut(&player_id).unwrap();

            current_state_player.nodes[0] += MOVEMENT_SPEED * (1.0 / TICK as f32);

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
