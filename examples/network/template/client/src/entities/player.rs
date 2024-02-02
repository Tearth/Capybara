use crate::network::game::GameNetworkContext;
use crate::scenes::GlobalData;
use capybara::app::ApplicationState;
use capybara::glam::Vec2;
use capybara::glam::Vec4;
use capybara::instant::Instant;
use capybara::renderer::shape::Shape;
use capybara::utils::color::Vec4Utils;
use network_template_base::game;
use network_template_base::game::GameState;

const INPUT_RESEND_INTERVAL: u32 = 500;

#[derive(Default)]
pub struct Player {
    pub heading_real: f32,
    pub heading_target: f32,
    pub nodes: Vec<Vec2>,

    pub last_cursor_position: Vec2,
    pub last_heading_update: Option<Instant>,

    pub initialized: bool,
}

impl Player {
    pub fn logic(&mut self, state: &mut ApplicationState<GlobalData>, network: &mut GameNetworkContext, delta: f32) {
        let now = Instant::now();

        if !self.initialized {
            if let Some(state) = &network.server_state {
                if let Some(player) = state.players.get(&network.player_id) {
                    self.nodes = player.nodes.to_vec();
                    self.initialized = true;
                }
            }

            return;
        }

        let camera = state.renderer.cameras.get_mut(state.renderer.active_camera_id).unwrap();
        let cursor_position = camera.from_window_to_world_coordinates(state.window.cursor_position.into());
        let heading_target = -(cursor_position - (self.nodes[0])).angle_between(Vec2::new(1.0, 0.0));
        let mut send_new_heading = cursor_position != self.last_cursor_position;

        if let Some(last_heading_update) = self.last_heading_update {
            if (now - last_heading_update).as_millis() > INPUT_RESEND_INTERVAL as u128 {
                send_new_heading = true;
            }
        } else {
            self.last_heading_update = Some(now);
        }

        if send_new_heading {
            network.send_new_heading(heading_target, now);
            self.last_cursor_position = cursor_position;
            self.last_heading_update = Some(Instant::now());
        }

        let result = game::simulate(GameState { nodes: self.nodes.clone(), heading_real: self.heading_real, heading_target }, delta);
        self.nodes = result.nodes;
        self.heading_real = result.heading_real;

        if !network.corrected_nodes.is_empty() {
            for i in 0..5 {
                let node_position = self.nodes[i];
                let corrected_node_position = network.corrected_nodes[i];
                let difference = corrected_node_position - node_position;

                self.nodes[i] += difference / 200.0;
            }
        }
    }

    pub fn draw(&mut self, state: &mut ApplicationState<GlobalData>, network: &mut GameNetworkContext) {
        if !self.initialized {
            return;
        }

        // Client-side predicted nodes
        for (index, node) in self.nodes.iter().enumerate() {
            let head_color = Vec4::new_rgb(255, 255, 255, 255);
            let body_color = Vec4::new_rgb(150, 150, 150, 255);
            let color = if index == 0 { head_color } else { body_color };

            state.renderer.draw_shape(&Shape::new_disc(*node, 20.0, None, color, color));
        }

        // Server-side nodes
        if let Some(network_state) = &network.server_state {
            if let Some(player) = network_state.players.get(&network.player_id) {
                for node in player.nodes {
                    let inner_color = Vec4::new_rgb(255, 180, 180, 255);
                    let outer_color = Vec4::new_rgb(255, 180, 180, 255);

                    state.renderer.draw_shape(&Shape::new_disc(node, 5.0, None, inner_color, outer_color));
                }
            }
        }

        // Server-side nodes with applied input
        for node in &network.corrected_nodes {
            let inner_color = Vec4::new_rgb(0, 0, 0, 255);
            let outer_color = Vec4::new_rgb(0, 0, 0, 255);

            state.renderer.draw_shape(&Shape::new_disc(*node, 5.0, None, inner_color, outer_color));
        }
    }
}
