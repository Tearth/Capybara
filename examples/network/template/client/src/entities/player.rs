use crate::network::game::GameNetworkContext;
use crate::scenes::GlobalData;
use capybara::app::ApplicationState;
use capybara::glam::Vec2;
use capybara::glam::Vec4;
use capybara::renderer::shape::Shape;
use capybara::utils::color::Vec4Utils;
use network_template_base::game;
use network_template_base::game::GameState;

#[derive(Default)]
pub struct Player {
    pub heading_real: f32,
    pub heading_target: f32,
    pub nodes: Vec<Vec2>,
    pub initialized: bool,
    pub last_cursor_position: Vec2,
}

impl Player {
    pub fn logic(&mut self, state: &mut ApplicationState<GlobalData>, network: &mut GameNetworkContext, delta: f32) {
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

        if cursor_position != self.last_cursor_position {
            network.send_new_heading(heading_target);
            self.last_cursor_position = cursor_position;
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

        for (index, node) in self.nodes.iter().enumerate() {
            let head_color = Vec4::new_rgb(255, 255, 255, 255);
            let body_color = Vec4::new_rgb(150, 150, 150, 255);

            state.renderer.draw_shape(&Shape::new_disc(
                *node,
                20.0,
                None,
                if index == 0 { head_color } else { body_color },
                if index == 0 { head_color } else { body_color },
            ));
        }

        if let Some(network_state) = &network.server_state {
            if let Some(player) = network_state.players.get(&network.player_id) {
                for (index, node) in player.nodes.iter().enumerate() {
                    let head_color = Vec4::new_rgb(255, 180, 180, 255);
                    let body_color = Vec4::new_rgb(255, 180, 180, 255);

                    state.renderer.draw_shape(&Shape::new_disc(
                        *node,
                        5.0,
                        None,
                        if index == 0 { head_color } else { body_color },
                        if index == 0 { head_color } else { body_color },
                    ));
                }
            }
        }

        for (index, node) in network.corrected_nodes.iter().enumerate() {
            let head_color = Vec4::new_rgb(0, 0, 0, 255);
            let body_color = Vec4::new_rgb(0, 0, 0, 255);

            state.renderer.draw_shape(&Shape::new_disc(
                *node,
                5.0,
                None,
                if index == 0 { head_color } else { body_color },
                if index == 0 { head_color } else { body_color },
            ));
        }
    }
}
