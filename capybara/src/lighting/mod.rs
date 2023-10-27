use glam::Vec2;

pub mod debug;
pub mod emitter;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EdgeWithDistance {
    pub a: Vec2,
    pub b: Vec2,
    pub distance: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayTarget {
    pub position: Vec2,
    pub angle: f32,
}

impl RayTarget {
    pub fn new(point: Vec2, angle: f32) -> Self {
        Self { position: point, angle }
    }
}
