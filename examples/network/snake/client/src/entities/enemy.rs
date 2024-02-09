use crate::network::game::GameNetworkContext;
use crate::network::game::ServerState;
use crate::scenes::GlobalData;
use capybara::app::ApplicationState;
use capybara::egui::ahash::HashMap;
use capybara::glam::Vec2;
use capybara::glam::Vec4;
use capybara::instant::Instant;
use capybara::renderer::shape::Shape;
use capybara::utils::color::Vec4Utils;
use std::collections::VecDeque;

pub const MINIMAL_STATES_COUNT: usize = 2;
pub const INTERPOLATION_BASE: f32 = 0.9;
pub const INTERPOLATION_INCREASE: f32 = 0.1;

#[derive(Default)]
pub struct Enemies {
    pub enemies: HashMap<u64, EnemyState>,
    pub states: VecDeque<ServerState>,
    pub state_change_timestamp: Option<Instant>,
}

#[derive(Default)]
pub struct EnemyState {
    pub nodes: Vec<Vec2>,
}

impl Enemies {
    pub fn logic(&mut self, network: &mut GameNetworkContext, now: Instant) {
        let mut state_update_timestamp = None;

        // Check if there is any new update from the server that needs to be synchronized
        if let Some(last_state) = self.states.back() {
            if let Some(last_server_state) = network.server_states.front() {
                if last_state.timestamp != last_server_state.timestamp {
                    state_update_timestamp = Some(last_state.timestamp);
                }
            }
        } else {
            if let Some(last_server_state) = network.server_states.front() {
                self.states.push_front(last_server_state.clone());
            }
        }

        // Synchronize with server state if needed
        if let Some(last_state_timestamp) = state_update_timestamp {
            for server_state in network.server_states.iter().rev() {
                if server_state.timestamp > last_state_timestamp {
                    self.states.push_back(server_state.clone());
                }
            }
        }

        if self.states.len() >= MINIMAL_STATES_COUNT {
            if self.state_change_timestamp.is_none() {
                self.state_change_timestamp = Some(now);
            }

            if let Some(state_change_timestamp) = self.state_change_timestamp {
                let current_state = &self.states[0];
                let future_state = &self.states[1];
                let interpolation_speed = INTERPOLATION_BASE + (self.states.len() - MINIMAL_STATES_COUNT) as f32 * INTERPOLATION_INCREASE;

                let delta = (future_state.timestamp - current_state.timestamp).as_millis();
                let alpha = ((now - state_change_timestamp).as_millis() as f32 / delta as f32) * interpolation_speed;

                for id in current_state.players.keys() {
                    if *id != network.player_id {
                        let current_player = match current_state.players.get(id) {
                            Some(nodes) => nodes,
                            None => continue,
                        };
                        let future_player = match future_state.players.get(id) {
                            Some(nodes) => nodes,
                            None => continue,
                        };

                        if let Some(enemy_state) = self.enemies.get_mut(id) {
                            for i in 0..enemy_state.nodes.len() {
                                enemy_state.nodes[i] = current_player.nodes[i] + (future_player.nodes[i] - current_player.nodes[i]) * alpha;
                            }
                        } else {
                            self.enemies.insert(*id, EnemyState { nodes: current_player.nodes.to_vec() });
                        }
                    }
                }

                if alpha >= 1.0 && self.states.len() > MINIMAL_STATES_COUNT {
                    self.states.pop_front();
                    self.state_change_timestamp = Some(now);

                    // Update nodes to the real ones, so there is no sudden jump caused by inconsistency
                    for (id, enemy) in &self.enemies {
                        if let Some(state) = self.states[0].players.get_mut(id) {
                            state.nodes.copy_from_slice(&enemy.nodes);
                        }
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, state: &mut ApplicationState<GlobalData>) {
        for enemy in &self.enemies {
            for (index, node) in enemy.1.nodes.iter().enumerate() {
                let head_color = Vec4::new_rgb(255, 255, 255, 255);
                let body_color = Vec4::new_rgb(150, 150, 150, 255);
                let color = if index == 0 { head_color } else { body_color };

                state.renderer.draw_shape(&Shape::new_disc(*node, 20.0, None, color, color));
            }
        }
    }
}
