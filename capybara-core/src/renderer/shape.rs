use glam::Mat4;
use glam::Vec2;
use glam::Vec3;

pub struct Shape {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub texture_id: Option<usize>,
    pub apply_model: bool,

    pub vertices: Vec<u32>,
    pub indices: Vec<u32>,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            position: Default::default(),
            rotation: 0.0,
            scale: Vec2::ONE,
            texture_id: None,
            apply_model: true,

            vertices: Default::default(),
            indices: Default::default(),
        }
    }

    pub fn get_model(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 0.0));

        translation * rotation * scale
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::new()
    }
}
