use glam::{Mat4, Vec2, Vec3};

pub struct Sprite {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Vec2,
    pub shape: Shape,
    pub texture_id: usize,
}

pub enum Shape {
    Standard,
    Custom(ShapeData),
}

pub struct ShapeData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

impl Sprite {
    pub fn new() -> Self {
        Self { position: Default::default(), rotation: 0.0, scale: Vec2::ONE, size: Default::default(), shape: Shape::Standard, texture_id: 0 }
    }

    pub fn get_model(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0))
            * Mat4::from_rotation_z(self.rotation)
            * Mat4::from_scale(Vec3::new(self.size.x * self.scale.x, self.size.y * self.scale.y, 0.0))
    }
}

impl ShapeData {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}
