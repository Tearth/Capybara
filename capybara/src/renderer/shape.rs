use super::sprite::TextureId;
use crate::renderer::lighting::Edge;
use crate::utils::color::Vec4Color;
use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::Vec4;
use std::f32::consts;

#[derive(Debug)]
pub struct Shape {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub texture_id: TextureId,
    pub apply_model: bool,

    pub vertices: Vec<ShapeVertex>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ShapeVertex {
    pub position: Vec2,
    pub color: u32,
    pub uv: Vec2,
}

impl Shape {
    pub fn new() -> Self {
        Self {
            position: Default::default(),
            rotation: 0.0,
            scale: Vec2::ONE,
            texture_id: TextureId::Default,
            apply_model: true,

            vertices: Default::default(),
            indices: Default::default(),
        }
    }

    pub fn new_line(from: Vec2, to: Vec2, thickness: f32, color: Vec4) -> Self {
        let width = thickness / 2.0;
        let length = (to - from).length() + 1.0;
        let angle = Vec2::new(0.0, 1.0).angle_between(to - from);

        let vertices = vec![
            // Left-bottom
            ShapeVertex::new(Vec2::new(-width, -0.5), color, Vec2::new(0.0, 1.0)),
            // Right-bottom
            ShapeVertex::new(Vec2::new(width, -0.5), color, Vec2::new(1.0, 1.0)),
            // Right-top
            ShapeVertex::new(Vec2::new(width, length - 0.5), color, Vec2::new(1.0, 0.0)),
            // Left-top
            ShapeVertex::new(Vec2::new(-width, length - 0.5), color, Vec2::new(0.0, 0.0)),
        ];

        Shape {
            position: from,
            rotation: angle,
            scale: Vec2::ONE,
            texture_id: TextureId::Default,
            apply_model: true,
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    pub fn new_rectangle(left_bottom: Vec2, right_top: Vec2, color: Vec4) -> Self {
        let size = right_top - left_bottom + Vec2::ONE;
        let vertices = vec![
            // Left-bottom
            ShapeVertex::new(left_bottom, color, Vec2::new(0.0, 1.0)),
            // Right-bottom
            ShapeVertex::new(left_bottom + Vec2::new(size.x, 0.0), color, Vec2::new(1.0, 1.0)),
            // Right-top
            ShapeVertex::new(left_bottom + size, color, Vec2::new(1.0, 0.0)),
            // Left-top
            ShapeVertex::new(left_bottom + Vec2::new(0.0, size.y), color, Vec2::new(0.0, 0.0)),
        ];

        Shape {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            texture_id: TextureId::Default,
            apply_model: true,
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    pub fn new_frame(left_bottom: Vec2, right_top: Vec2, thickness: f32, color: Vec4) -> Self {
        let size = right_top - left_bottom + Vec2::ONE;
        let uv_thickness = thickness / size;
        let vertices = vec![
            // Left-bottom outer
            ShapeVertex::new(left_bottom, color, Vec2::new(0.0, 1.0)),
            // Left-bottom inner
            ShapeVertex::new(left_bottom + thickness, color, Vec2::new(uv_thickness.x, 1.0 - uv_thickness.y)),
            // Right-bottom outer
            ShapeVertex::new(left_bottom + Vec2::new(size.x, 0.0), color, Vec2::new(1.0, 1.0)),
            // Right-bottom inner
            ShapeVertex::new(left_bottom + Vec2::new(size.x - thickness, thickness), color, Vec2::new(1.0, 1.0) - uv_thickness),
            // Right-top outer
            ShapeVertex::new(left_bottom + size, color, Vec2::new(1.0, 0.0)),
            // Right-top inner
            ShapeVertex::new(left_bottom + size - thickness, color, Vec2::new(1.0 - uv_thickness.x, uv_thickness.y)),
            // Left-top outer
            ShapeVertex::new(left_bottom + Vec2::new(0.0, size.y), color, Vec2::new(0.0, 0.0)),
            // Left-top inner
            ShapeVertex::new(left_bottom + Vec2::new(thickness, size.y - thickness), color, uv_thickness),
        ];

        Shape {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            texture_id: TextureId::Default,
            apply_model: true,
            vertices,
            indices: vec![0, 2, 1, 2, 3, 1, 2, 4, 5, 2, 5, 3, 4, 6, 5, 6, 7, 5, 6, 0, 1, 6, 1, 7],
        }
    }

    pub fn new_disc(center: Vec2, radius: f32, sides: Option<u32>, inner_color: Vec4, outer_color: Vec4) -> Self {
        let sides = sides.unwrap_or((radius * 4.0) as u32);
        let angle_step = consts::TAU / sides as f32;
        let mut vertices = vec![
            // Center
            ShapeVertex::new(center, inner_color, Vec2::new(0.5, 0.5)),
        ];
        let mut indices = Vec::new();

        for i in 0..sides {
            let angle = i as f32 * angle_step;
            let sin = f32::sin(angle);
            let cos = f32::cos(angle);

            let position = Vec2::new(sin * radius + center.x, cos * radius + center.y);
            let uv = Vec2::new(sin / 2.0 + 0.5, 1.0 - (cos / 2.0 + 0.5));
            vertices.push(ShapeVertex::new(position, outer_color, uv));

            if i > 0 {
                indices.extend_from_slice(&[0, i, i + 1]);
            }
        }

        indices.extend_from_slice(&[0, sides, 1]);

        Shape { position: Vec2::ZERO, rotation: 0.0, scale: Vec2::ONE, texture_id: TextureId::Default, apply_model: true, vertices, indices }
    }

    pub fn new_circle(center: Vec2, radius: f32, sides: Option<u32>, thickness: f32, color: Vec4) -> Self {
        let sides = sides.unwrap_or((radius * 4.0) as u32);
        let uv_thickness = thickness / radius;
        let angle_step = consts::TAU / sides as f32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..sides {
            let angle = i as f32 * angle_step;
            let sin = f32::sin(angle);
            let cos = f32::cos(angle);

            let position_outer = Vec2::new(sin * radius + center.x, cos * radius + center.y);
            let position_inner = Vec2::new(sin * (radius - thickness) + center.x, cos * (radius - thickness) + center.y);
            let uv_outer = Vec2::new(sin / 2.0 + 0.5, 1.0 - (cos / 2.0 + 0.5));
            let uv_inner = Vec2::new(sin * (1.0 - uv_thickness) / 2.0 + 0.5, 1.0 - (cos * (1.0 - uv_thickness) / 2.0 + 0.5));

            vertices.push(ShapeVertex::new(position_outer, color, uv_outer));
            vertices.push(ShapeVertex::new(position_inner, color, uv_inner));

            if i > 0 {
                let i = i * 2;
                indices.extend_from_slice(&[i - 2, i - 1, i + 1, i - 2, i, i + 1]);
            }
        }

        let i = sides * 2;
        indices.extend_from_slice(&[i - 2, i - 1, 1, i - 2, 0, 1]);

        Shape { position: Vec2::ZERO, rotation: 0.0, scale: Vec2::ONE, texture_id: TextureId::Default, apply_model: true, vertices, indices }
    }

    pub fn get_edges(&self) -> Vec<Edge> {
        let mut edges = Vec::new();
        let model = self.get_model();

        for i in (0..self.indices.len()).step_by(3) {
            let a = self.vertices[self.indices[i + 0] as usize].position;
            let b = self.vertices[self.indices[i + 1] as usize].position;
            let c = self.vertices[self.indices[i + 2] as usize].position;

            let a = model * Vec4::new(a.x, a.y, 0.0, 1.0);
            let b = model * Vec4::new(b.x, b.y, 0.0, 1.0);
            let c = model * Vec4::new(c.x, c.y, 0.0, 1.0);

            let a = Vec2::new(a.x, a.y);
            let b = Vec2::new(b.x, b.y);
            let c = Vec2::new(c.x, c.y);

            edges.extend_from_slice(&[Edge::new(a, b), Edge::new(b, c), Edge::new(c, a)]);
        }

        edges
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

impl ShapeVertex {
    pub fn new(position: Vec2, color: Vec4, uv: Vec2) -> Self {
        Self { position, color: color.to_rgb_packed(), uv }
    }
}
