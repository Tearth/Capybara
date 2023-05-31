use glam::Vec2;
use glam::Vec4;

pub struct Shape {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Vec2,
    pub anchor: Vec2,
    pub color: Vec4,
    pub texture_id: usize,

    pub vertices: Vec<u32>,
    pub indices: Vec<u32>,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            position: Default::default(),
            rotation: 0.0,
            scale: Vec2::ONE,
            size: Default::default(),
            anchor: Vec2::new(0.5, 0.5),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            texture_id: 0,

            vertices: Default::default(),
            indices: Default::default(),
        }
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::new()
    }
}
