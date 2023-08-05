use crate::utils::color::Vec4Color;
use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::Vec4;

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

    pub fn new_line(from: Vec2, to: Vec2, thickness: f32, color: Vec4) -> Self {
        let width = thickness / 2.0;
        let length = (to - from).length() + 1.0;
        let angle = Vec2::new(0.0, 1.0).angle_between(to - from);
        let mut vertices = Vec::new();

        let color = color.to_rgb_packed();

        vertices.push((-width).to_bits());
        vertices.push((-0.5f32).to_bits());
        vertices.push(color);
        vertices.push(0.0f32.to_bits());
        vertices.push(0.0f32.to_bits());

        vertices.push((width).to_bits());
        vertices.push((-0.5f32).to_bits());
        vertices.push(color);
        vertices.push(1.0f32.to_bits());
        vertices.push(0.0f32.to_bits());

        vertices.push((width).to_bits());
        vertices.push((length - 0.5).to_bits());
        vertices.push(color);
        vertices.push(1.0f32.to_bits());
        vertices.push(1.0f32.to_bits());

        vertices.push((-width).to_bits());
        vertices.push((length - 0.5).to_bits());
        vertices.push(color);
        vertices.push(0.0f32.to_bits());
        vertices.push(1.0f32.to_bits());

        Shape {
            position: from + Vec2::new(0.5, 0.5),
            rotation: angle,
            scale: Vec2::ONE,
            texture_id: None,
            apply_model: true,
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
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
