use glam::Vec2;

pub mod camera;
pub mod context;
pub mod particles;
pub mod shader;
pub mod shape;
pub mod sprite;
pub mod texture;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Edge {
    pub a: Vec2,
    pub b: Vec2,
}

impl Edge {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }
}
