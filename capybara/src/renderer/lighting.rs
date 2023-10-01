use super::context::RendererContext;
use super::shape::Shape;
use super::*;
use crate::utils::color::Vec4Color;
use glam::Vec4;
use std::f32::consts;

pub struct LightEmitter {
    pub position: Vec2,
    pub edges: Vec<Edge>,
    pub offsets: Vec<f32>,
    pub color: Vec4,
    pub max_length: f32,

    pub debug: bool,
    pub debug_settings: LightDebugSettings,
}

pub struct LightDebugSettings {
    pub edge_color: Vec4,
    pub ray_color: Vec4,
    pub point_color: Vec4,
    pub hit_color: Vec4,

    pub edge_thickness: f32,
    pub ray_thickness: f32,
    pub point_radius: f32,
    pub hit_radius: f32,
}

pub struct LightResponse {
    pub shape: Shape,
    pub points: Vec<LightRayTarget>,
    pub hits: Vec<LightRayTarget>,
}

pub struct LightRayTarget {
    pub position: Vec2,
    pub angle: f32,
}

impl LightEmitter {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            edges: Vec::new(),
            offsets: vec![-0.002, 0.0, 0.002],
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            max_length: 200.0,

            debug: false,
            debug_settings: LightDebugSettings {
                edge_color: Vec4::new(1.0, 1.0, 0.0, 1.0),
                ray_color: Vec4::new(1.0, 1.0, 0.0, 1.0),
                point_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                hit_color: Vec4::new(1.0, 1.0, 1.0, 1.0),

                edge_thickness: 1.0,
                ray_thickness: 1.0,
                point_radius: 5.0,
                hit_radius: 3.0,
            },
        }
    }

    pub fn generate(&self) -> LightResponse {
        // Algorithm based on:
        // - https://ncase.me/sight-and-light/
        // - https://rootllama.wordpress.com/2014/06/20/ray-line-segment-intersection-test-in-2d/

        // -----------------------------------------------------------------------------------
        // Step 1: iterate through all edges and collect points toward which rays will be cast
        // -----------------------------------------------------------------------------------

        let mut points = Vec::new();
        let mut hits = Vec::new();

        for edge in &self.edges {
            for offset in &self.offsets {
                let angle_a = Vec2::new(0.0, 1.0).angle_between(self.position - edge.a) - consts::PI / 2.0;
                let angle_b = Vec2::new(0.0, 1.0).angle_between(self.position - edge.b) - consts::PI / 2.0;

                if angle_a.is_nan() || angle_b.is_nan() {
                    continue;
                }

                points.push(LightRayTarget::new(edge.a, angle_a + offset));
                points.push(LightRayTarget::new(edge.b, angle_b + offset));
            }
        }

        // ----------------------------------------------------------------------------------------
        // Step 2: sort and deduplicate points, so the mesh can be later generated in correct order
        // ----------------------------------------------------------------------------------------

        points.sort_by(|a, b| a.angle.partial_cmp(&b.angle).unwrap());
        points.dedup_by(|a, b| a.angle == b.angle);

        // ----------------------------------------------------------------------------------------------
        // Step 3: calculate points of hits between rays and edges, select nearest ones and put into list
        // ----------------------------------------------------------------------------------------------

        for point in &points {
            // Ray  = pa + da * ta, 0.0 <= ta
            // Edge = pb + db * tb, 0.0 <= tb <= 1.0

            let pa = self.position;
            let da = Vec2::from_angle(point.angle);
            let mut smallest_ta = f32::MAX;

            for edge in &self.edges {
                let pb = edge.a;
                let db = edge.b - edge.a;

                let tb = (da.x * (pb.y - pa.y) - da.y * (pb.x - pa.x)) / (da.y * db.x - da.x * db.y);
                let ta = (pb.x + db.x * tb - pa.x) / da.x;

                if ta >= 0.0 && tb >= 0.0 && tb <= 1.0 {
                    if ta < smallest_ta {
                        smallest_ta = ta;
                    }
                }
            }

            if smallest_ta != f32::MAX && smallest_ta != 0.0 {
                hits.push(LightRayTarget::new(pa + da * smallest_ta.min(self.max_length), point.angle));
            }
        }

        // -----------------------------------------------------------------------------------
        // Step 4: generate mesh with first vertice centered and all others placed circle-like
        // -----------------------------------------------------------------------------------

        let mut shape = Shape::new();
        let color = self.color.to_rgb_packed();

        shape.add_vertice(self.position, color, Vec2::new(0.0, 0.0));
        for (index, hit) in hits.iter().enumerate() {
            shape.add_vertice((*hit).position, color, Vec2::new(1.0, 1.0));
            if index > 0 {
                shape.indices.push(0);
                shape.indices.push(index as u32);
                shape.indices.push(index as u32 + 1);
            }
        }

        shape.indices.push(0);
        shape.indices.push(hits.len() as u32);
        shape.indices.push(1);

        LightResponse { shape, points: if self.debug { points } else { Vec::new() }, hits: if self.debug { hits } else { Vec::new() } }
    }

    pub fn draw_debug(&self, renderer: &mut RendererContext, response: &LightResponse) {
        for edge in &self.edges {
            renderer.draw_shape(&Shape::new_line(edge.a, edge.b, self.debug_settings.edge_thickness, self.debug_settings.edge_color));
        }

        for point in &response.points {
            let p = self.position;
            let d = Vec2::from_angle(point.angle);

            renderer.draw_shape(&Shape::new_line(p, p + d * 10000.0, self.debug_settings.ray_thickness, self.debug_settings.ray_color));
            renderer.draw_shape(&Shape::new_disc(point.position, self.debug_settings.point_radius, None, self.debug_settings.point_color));
        }

        for hit in &response.hits {
            renderer.draw_shape(&Shape::new_disc((*hit).position, self.debug_settings.hit_radius, None, self.debug_settings.hit_color));
        }
    }
}

impl Default for LightEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl LightRayTarget {
    pub fn new(point: Vec2, angle: f32) -> Self {
        Self { position: point, angle }
    }
}
