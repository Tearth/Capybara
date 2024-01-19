use super::emitter::LightEmitter;
use super::emitter::LightResponse;
use crate::renderer::context::RendererContext;
use crate::renderer::shape::Shape;
use crate::renderer::Edge;
use glam::Vec2;
use glam::Vec4;
use std::f32::consts;

#[derive(Debug)]
pub struct LightDebugSettings {
    pub enabled: bool,

    pub edge_color: Vec4,
    pub ray_color: Vec4,
    pub arc_color: Vec4,
    pub point_color: Vec4,
    pub hit_color: Vec4,

    pub edge_thickness: f32,
    pub ray_thickness: f32,
    pub arc_thickness: f32,
    pub point_radius: f32,
    pub hit_radius: f32,
}

impl Default for LightDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,

            edge_color: Vec4::new(1.0, 1.0, 0.0, 1.0),
            ray_color: Vec4::new(1.0, 1.0, 0.0, 1.0),
            arc_color: Vec4::new(1.0, 0.0, 1.0, 1.0),
            point_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            hit_color: Vec4::new(1.0, 1.0, 1.0, 1.0),

            edge_thickness: 1.0,
            ray_thickness: 1.0,
            arc_thickness: 1.0,
            point_radius: 5.0,
            hit_radius: 3.0,
        }
    }
}

impl LightEmitter {
    pub fn draw_debug(&self, renderer: &mut RendererContext, data: &[Edge], response: &LightResponse) {
        for edge in data {
            renderer.draw_shape(&Shape::new_line(edge.a, edge.b, self.debug.edge_thickness, self.debug.edge_color));
        }

        for point in &response.points {
            let p = self.position;
            let d = Vec2::from_angle(point.angle);

            renderer.draw_shape(&Shape::new_line(p, p + d * self.max_length, self.debug.ray_thickness, self.debug.ray_color));
            renderer.draw_shape(&Shape::new_disc(point.position, self.debug.point_radius, None, self.debug.point_color, self.debug.point_color));
        }

        for hit in &response.hits {
            renderer.draw_shape(&Shape::new_disc(hit.position, self.debug.hit_radius, None, self.debug.hit_color, self.debug.hit_color));
        }

        if self.arc < consts::TAU {
            let p = self.position;
            let d1 = Vec2::from_angle(self.angle - self.arc / 2.0);
            let d2 = Vec2::from_angle(self.angle + self.arc / 2.0);

            renderer.draw_shape(&Shape::new_line(p, p + d1 * self.max_length, self.debug.arc_thickness, self.debug.arc_color));
            renderer.draw_shape(&Shape::new_line(p, p + d2 * self.max_length, self.debug.arc_thickness, self.debug.arc_color));
        }
    }
}
