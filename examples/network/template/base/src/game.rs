use crate::*;
use capybara::glam::Vec2;
use std::f32::consts;

#[derive(Default)]
pub struct GameState {
    pub nodes: Vec<Vec2>,
    pub heading_real: f32,
    pub heading_target: f32,
}

pub fn simulate(state: GameState, delta: f32) -> GameState {
    let mut heading_updated = None;
    let mut state_update = GameState::default();

    state_update.nodes = state.nodes;
    state_update.heading_target = state.heading_target;

    let mut heading_difference = (state.heading_target - state.heading_real + consts::PI) % consts::TAU - consts::PI;
    heading_difference = if heading_difference < -consts::PI { heading_difference + consts::TAU } else { heading_difference };

    if heading_difference != 0.0 {
        if heading_difference > consts::PI {
            heading_difference = consts::TAU - heading_difference;
        }

        let rotation_ratio = ROTATION_SPEED * delta;
        if rotation_ratio >= heading_difference.abs() {
            heading_updated = Some(state.heading_target);
        } else {
            heading_updated = Some(state.heading_real + heading_difference.signum() * rotation_ratio);
        }
    }

    state_update.heading_real = heading_updated.unwrap_or(state.heading_real);
    state_update.nodes[0] += Vec2::from_angle(state_update.heading_real) * MOVEMENT_SPEED * delta;

    for node_index in 1..state_update.nodes.len() {
        let current_node = state_update.nodes[node_index];
        let parent_node = state_update.nodes[node_index - 1];
        let direction = (parent_node - current_node).normalize();

        state_update.nodes[node_index] = parent_node - direction * DISTANCE_BETWEEN_NODES;
    }

    state_update
}
