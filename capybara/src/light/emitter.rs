use super::debug::LightDebugSettings;
use super::*;
use crate::renderer::shape::Shape;
use crate::renderer::shape::ShapeVertex;
use crate::renderer::Edge;
use crate::utils::color::Vec4Color;
use crate::utils::math::F32MathUtils;
use crate::utils::math::Vec2MathUtils;
use glam::Vec2;
use glam::Vec4;
use std::f32::consts;

pub struct LightEmitter {
    pub position: Vec2,
    pub offset: f32,
    pub color_begin: Vec4,
    pub color_end: Vec4,
    pub angle: f32,
    pub arc: f32,
    pub max_length: f32,
    pub frame_rays: u32,
    pub merge_distance: f32,
    pub tolerance: f32,
    pub extension: f32,
    pub debug: LightDebugSettings,

    edges: Vec<EdgeWithDistance>,
    points: Vec<RayTarget>,
    hits: Vec<RayTarget>,
}

#[derive(Debug, Default)]
pub struct LightResponse {
    pub shape: Shape,
    pub points: Vec<RayTarget>,
    pub hits: Vec<RayTarget>,
}

impl LightEmitter {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            offset: 0.002,
            color_begin: Vec4::new(1.0, 1.0, 1.0, 1.0),
            color_end: Vec4::new(1.0, 1.0, 1.0, 1.0),
            angle: 0.0,
            arc: consts::TAU,
            max_length: 10000.0,
            frame_rays: 32,
            merge_distance: 1.0,
            tolerance: 0.0001,
            extension: 0.0,
            debug: LightDebugSettings::default(),

            edges: Vec::new(),
            points: Vec::new(),
            hits: Vec::new(),
        }
    }

    pub fn generate(&mut self, data: &[Edge]) -> LightResponse {
        // Algorithm based on:
        // - https://ncase.me/sight-and-light/
        // - https://rootllama.wordpress.com/2014/06/20/ray-line-segment-intersection-test-in-2d/

        self.edges.clear();
        self.points.clear();
        self.hits.clear();

        // -------------------------------------------------------
        // Step 1: collect points based on angle boundaries if set
        // -------------------------------------------------------

        // Do not calculate angles if arc is TAU, or else angle_from will be PI in some cases due to calculation errors
        let (angle_from, mut angle_to) = if self.arc < consts::TAU {
            ((self.angle - self.arc / 2.0).normalize_angle(), (self.angle + self.arc / 2.0).normalize_angle())
        } else {
            (-consts::PI, consts::PI)
        };

        // Normalize order if desired angle is between PI and -PI (sign changes backwards)
        if angle_from > angle_to {
            angle_to += consts::TAU;
        }

        if self.arc < consts::TAU {
            let p = self.position;
            let d1 = Vec2::from_angle(angle_from);
            let d2 = Vec2::from_angle(angle_to);

            self.points.push(RayTarget::new(p + d1 * self.max_length, angle_from));
            self.points.push(RayTarget::new(p + d2 * self.max_length, angle_to));
        }

        // ----------------------------------------------------------------------------------
        // Step 2: collect points based on frame rays to maintain overall shape of light mesh
        // ----------------------------------------------------------------------------------

        if self.frame_rays > 0 {
            let p = self.position;
            let step = self.arc / self.frame_rays as f32;

            for i in 0..self.frame_rays {
                let a = angle_from + (i as f32 * step);
                let d = Vec2::from_angle(a);

                self.points.push(RayTarget::new(p + d * self.max_length, a));
            }
        }

        // -------------------------------------------------------------------------------------
        // Step 3: sort edges by distance from the position so the search can be optimized later
        // -------------------------------------------------------------------------------------

        for edge in data {
            let distance = self.position.distance_to_segment(edge.a, edge.b);
            if distance <= self.max_length {
                self.edges.push(EdgeWithDistance { a: edge.a, b: edge.b, distance });
            }
        }

        self.edges.sort_unstable_by(|a, b| unsafe { a.distance.partial_cmp(&b.distance).unwrap_unchecked() });

        // ----------------------------------------------------------------------------
        // Step 4: add the most outer edges, so every ray will eventually hit something
        // ----------------------------------------------------------------------------

        let outer_edge_distance = self.max_length * 2.0;
        let left_bottom = self.position + outer_edge_distance * Vec2::new(-1.0, -1.0);
        let right_bottom = self.position + outer_edge_distance * Vec2::new(1.0, -1.0);
        let right_top = self.position + outer_edge_distance * Vec2::new(1.0, 1.0);
        let left_top = self.position + outer_edge_distance * Vec2::new(-1.0, 1.0);

        self.edges.push(EdgeWithDistance { a: left_bottom, b: right_bottom, distance: outer_edge_distance });
        self.edges.push(EdgeWithDistance { a: right_bottom, b: right_top, distance: outer_edge_distance });
        self.edges.push(EdgeWithDistance { a: right_top, b: left_top, distance: outer_edge_distance });
        self.edges.push(EdgeWithDistance { a: left_top, b: left_bottom, distance: outer_edge_distance });

        // ----------------------------------------------------
        // Step 5: iterate through all edges and collect points
        // ----------------------------------------------------

        for edge in &self.edges {
            for offset in [-self.offset, 0.0, self.offset] {
                let angle_a = Vec2::new(0.0, 1.0).angle_between(self.position - edge.a) - consts::FRAC_PI_2 + offset;
                let angle_b = Vec2::new(0.0, 1.0).angle_between(self.position - edge.b) - consts::FRAC_PI_2 + offset;

                if angle_a.is_nan() || angle_b.is_nan() {
                    continue;
                }

                let mut angle_a = angle_a.normalize_angle();
                let mut angle_b = angle_b.normalize_angle();

                // Normalize order if desired angle is between PI and -PI (sign changes backwards)
                if self.arc < consts::TAU {
                    if angle_a < angle_from {
                        angle_a += consts::TAU;
                    }
                    if angle_b < angle_from {
                        angle_b += consts::TAU;
                    }
                }

                if self.arc == consts::TAU || (angle_a >= angle_from && angle_a <= angle_to) {
                    self.points.push(RayTarget::new(edge.a, angle_a));
                }
                if self.arc == consts::TAU || (angle_b >= angle_from && angle_b <= angle_to) {
                    self.points.push(RayTarget::new(edge.b, angle_b));
                }
            }
        }

        // ----------------------------------------------------------------------------------------
        // Step 6: sort and deduplicate points, so the mesh can be later generated in correct order
        // ----------------------------------------------------------------------------------------

        self.points.sort_by(|a, b| unsafe { a.angle.partial_cmp(&b.angle).unwrap_unchecked() });
        self.points.dedup_by(|a, b| a.angle == b.angle);

        // ----------------------------------------------------------------------------------------------
        // Step 7: calculate points of hits between rays and edges, select nearest ones and put into list
        // ----------------------------------------------------------------------------------------------

        for point in &self.points {
            // Ray  = pa + da * ta, 0.0 < ta
            // Edge = pb + db * tb, 0.0 < tb < 1.0

            let pa = self.position;
            let da = Vec2::from_angle(point.angle);
            let mut ta_min = f32::MAX;

            for edge in &self.edges {
                // Edges are sorted, so do not search these with larger distance than the found hit
                if ta_min != f32::MAX && edge.distance > ta_min + self.tolerance {
                    break;
                }

                let pb = edge.a;
                let db = edge.b - edge.a;

                let ta = (db.x * (pa.y - pb.y) - db.y * (pa.x - pb.x)) / (db.y * da.x - db.x * da.y);
                let tb = (da.x * (pb.y - pa.y) - da.y * (pb.x - pa.x)) / (da.y * db.x - da.x * db.y);

                if ta > 0.0 && tb > -self.tolerance && tb < 1.0 + self.tolerance && ta < ta_min {
                    ta_min = ta;
                }
            }

            if ta_min != f32::MAX {
                self.hits.push(RayTarget::new(pa + da * (ta_min + self.extension).min(self.max_length), point.angle));
            }
        }

        // -----------------------------------------------------------------------------------
        // Step 8: generate mesh with first vertex centered and all others placed circle-like
        // -----------------------------------------------------------------------------------

        let mut shape = Shape::new();
        let mut last_position = Vec2::new(0.0, 0.0);

        shape.vertices.push(ShapeVertex::new(self.position, self.color_begin.to_rgb_packed(), Vec2::new(0.0, 0.0)));
        for hit in &self.hits {
            if hit.position.distance(last_position) <= self.merge_distance {
                continue;
            }

            let index = shape.vertices.len() - 1;
            let distance = hit.position.distance(self.position);
            let ratio = distance / self.max_length;
            let color = self.color_begin.lerp(self.color_end, ratio);

            shape.vertices.push(ShapeVertex::new(hit.position, color.to_rgb_packed(), Vec2::new(1.0, 1.0)));
            if index > 0 {
                shape.indices.push(0);
                shape.indices.push(index as u32);
                shape.indices.push(index as u32 + 1);
            }

            last_position = hit.position;
        }

        if self.arc == consts::TAU {
            shape.indices.push(0);
            shape.indices.push((shape.vertices.len() - 1) as u32);
            shape.indices.push(1);
        }

        LightResponse {
            shape,
            points: if self.debug.enabled { self.points.clone() } else { Vec::new() },
            hits: if self.debug.enabled { self.hits.clone() } else { Vec::new() },
        }
    }
}

impl Default for LightEmitter {
    fn default() -> Self {
        Self::new()
    }
}
