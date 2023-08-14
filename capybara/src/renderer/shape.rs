use crate::utils::color::Vec4Color;
use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::Vec4;
use std::f32::consts;

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
        let color = color.to_rgb_packed();

        let vertices = vec![
            // Left-bottom
            (-width).to_bits(),
            (-0.5f32).to_bits(),
            color,
            0.0f32.to_bits(),
            1.0f32.to_bits(),
            // Right-bottom
            (width).to_bits(),
            (-0.5f32).to_bits(),
            color,
            1.0f32.to_bits(),
            1.0f32.to_bits(),
            // Right-top
            (width).to_bits(),
            (length - 0.5).to_bits(),
            color,
            1.0f32.to_bits(),
            0.0f32.to_bits(),
            // Left-top
            (-width).to_bits(),
            (length - 0.5).to_bits(),
            color,
            0.0f32.to_bits(),
            0.0f32.to_bits(),
        ];

        Shape { position: from, rotation: angle, scale: Vec2::ONE, texture_id: None, apply_model: true, vertices, indices: vec![0, 1, 2, 0, 2, 3] }
    }

    pub fn new_rectangle(left_bottom: Vec2, right_top: Vec2, color: Vec4) -> Self {
        let size = right_top - left_bottom + Vec2::ONE;
        let color = color.to_rgb_packed();
        let vertices = vec![
            // Left-bottom
            (left_bottom.x).to_bits(),
            (left_bottom.y).to_bits(),
            color,
            0.0f32.to_bits(),
            1.0f32.to_bits(),
            // Right-bottom
            (left_bottom.x + size.x).to_bits(),
            (left_bottom.y).to_bits(),
            color,
            1.0f32.to_bits(),
            1.0f32.to_bits(),
            // Right-top
            (left_bottom.x + size.x).to_bits(),
            (left_bottom.y + size.y).to_bits(),
            color,
            1.0f32.to_bits(),
            0.0f32.to_bits(),
            // Left-top
            (left_bottom.x).to_bits(),
            (left_bottom.y + size.y).to_bits(),
            color,
            0.0f32.to_bits(),
            0.0f32.to_bits(),
        ];

        Shape {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            texture_id: None,
            apply_model: true,
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    pub fn new_frame(left_bottom: Vec2, right_top: Vec2, thickness: f32, color: Vec4) -> Self {
        let size = right_top - left_bottom + Vec2::ONE;
        let color = color.to_rgb_packed();
        let uv_thickness = thickness / size;
        let vertices = vec![
            // Left-bottom outer
            (left_bottom.x).to_bits(),
            (left_bottom.y).to_bits(),
            color,
            0.0f32.to_bits(),
            1.0f32.to_bits(),
            // Left-bottom inner
            (left_bottom.x + thickness).to_bits(),
            (left_bottom.y + thickness).to_bits(),
            color,
            (0.0 + uv_thickness.x).to_bits(),
            (1.0 - uv_thickness.y).to_bits(),
            // Right-bottom outer
            (left_bottom.x + size.x).to_bits(),
            (left_bottom.y).to_bits(),
            color,
            1.0f32.to_bits(),
            1.0f32.to_bits(),
            // Right-bottom inner
            (left_bottom.x + size.x - thickness).to_bits(),
            (left_bottom.y + thickness).to_bits(),
            color,
            (1.0 - uv_thickness.x).to_bits(),
            (1.0 - uv_thickness.y).to_bits(),
            // Right-top outer
            (left_bottom.x + size.x).to_bits(),
            (left_bottom.y + size.y).to_bits(),
            color,
            1.0f32.to_bits(),
            0.0f32.to_bits(),
            // Right-top inner
            (left_bottom.x + size.x - thickness).to_bits(),
            (left_bottom.y + size.y - thickness).to_bits(),
            color,
            (1.0 - uv_thickness.x).to_bits(),
            (0.0 + uv_thickness.y).to_bits(),
            // Left-top outer
            (left_bottom.x).to_bits(),
            (left_bottom.y + size.y).to_bits(),
            color,
            0.0f32.to_bits(),
            0.0f32.to_bits(),
            // Left-top inner
            (left_bottom.x + thickness).to_bits(),
            (left_bottom.y + size.y - thickness).to_bits(),
            color,
            (0.0 + uv_thickness.x).to_bits(),
            (0.0 + uv_thickness.y).to_bits(),
        ];

        Shape {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            texture_id: None,
            apply_model: true,
            vertices,
            indices: vec![0, 2, 1, 2, 3, 1, 2, 4, 5, 2, 5, 3, 4, 6, 5, 6, 7, 5, 6, 0, 1, 6, 1, 7],
        }
    }

    pub fn new_disc(center: Vec2, radius: f32, sides: Option<u32>, color: Vec4) -> Self {
        let sides = sides.unwrap_or((radius * 4.0) as u32);
        let color = color.to_rgb_packed();
        let angle_step = consts::TAU / sides as f32;
        let mut vertices = vec![
            // Center
            center.x.to_bits(),
            center.y.to_bits(),
            color,
            0.5f32.to_bits(),
            0.5f32.to_bits(),
        ];
        let mut indices = Vec::new();

        for i in 0..sides {
            let angle = i as f32 * angle_step;
            let sin = f32::sin(angle);
            let cos = f32::cos(angle);

            vertices.extend_from_slice(&[
                (sin * radius + center.x).to_bits(),
                (cos * radius + center.y).to_bits(),
                color,
                (sin / 2.0 + 0.5).to_bits(),
                (1.0 - (cos / 2.0 + 0.5)).to_bits(),
            ]);

            if i > 0 {
                indices.extend_from_slice(&[0, i, i + 1]);
            }
        }

        indices.extend_from_slice(&[0, sides, 1]);

        Shape { position: Vec2::ZERO, rotation: 0.0, scale: Vec2::ONE, texture_id: None, apply_model: true, vertices, indices }
    }

    pub fn new_circle(center: Vec2, radius: f32, sides: Option<u32>, thickness: f32, color: Vec4) -> Self {
        let sides = sides.unwrap_or((radius * 4.0) as u32);
        let color = color.to_rgb_packed();
        let uv_thickness = thickness / radius;
        let angle_step = consts::TAU / sides as f32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..sides {
            let angle = i as f32 * angle_step;
            let sin = f32::sin(angle);
            let cos = f32::cos(angle);

            vertices.extend_from_slice(&[
                // Outer
                (sin * radius + center.x).to_bits(),
                (cos * radius + center.y).to_bits(),
                color,
                (sin / 2.0 + 0.5).to_bits(),
                (1.0 - (cos / 2.0 + 0.5)).to_bits(),
                // Inner
                (sin * (radius - thickness) + center.x).to_bits(),
                (cos * (radius - thickness) + center.y).to_bits(),
                color,
                (sin * (1.0 - uv_thickness) / 2.0 + 0.5).to_bits(),
                (1.0 - (cos * (1.0 - uv_thickness) / 2.0 + 0.5)).to_bits(),
            ]);

            if i > 0 {
                let i = i * 2;
                indices.extend_from_slice(&[i - 2, i - 1, i + 1, i - 2, i, i + 1]);
            }
        }

        let i = sides * 2;
        indices.extend_from_slice(&[i - 2, i - 1, 1, i - 2, 0, 1]);

        Shape { position: Vec2::ZERO, rotation: 0.0, scale: Vec2::ONE, texture_id: None, apply_model: true, vertices, indices }
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
