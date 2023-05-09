use glam::{Mat4, Vec2, Vec3};

pub struct Sprite {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Vec2,
}

impl Sprite {
    pub fn new() -> Self {
        Self { position: Default::default(), rotation: 0.0, scale: Vec2::ONE, size: Default::default() }
    }

    pub fn get_model(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0))
            * Mat4::from_rotation_z(self.rotation)
            * Mat4::from_scale(Vec3::new(self.size.x * self.scale.x, self.size.y * self.scale.y, 0.0))
    }
}
