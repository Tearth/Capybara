use capybara::glam::Vec2;

pub mod simulation;

#[derive(Debug, Default)]
pub struct GameState {
    pub nodes: Vec<Vec2>,
    pub heading_real: f32,
    pub heading_target: f32,
}
