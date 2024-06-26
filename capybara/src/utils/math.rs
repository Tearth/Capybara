use glam::Vec2;
use std::f32::consts;

pub trait F32Utils {
    fn normalize_angle(&self) -> f32;
}

pub trait Vec2Utils {
    fn distance_to_line(&self, a: Vec2, b: Vec2) -> f32;
    fn distance_to_segment(&self, a: Vec2, b: Vec2) -> f32;
}

impl F32Utils for f32 {
    fn normalize_angle(&self) -> f32 {
        if *self < -consts::PI {
            *self + consts::TAU
        } else {
            *self
        }
    }
}

impl Vec2Utils for Vec2 {
    fn distance_to_line(&self, a: Vec2, b: Vec2) -> f32 {
        let x = ((b.x - a.x) * (a.y - self.y) - (a.x - self.x) * (b.y - a.y)).abs();
        let y = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();

        x / y
    }

    fn distance_to_segment(&self, a: Vec2, b: Vec2) -> f32 {
        let ab = b - a;
        let ap = *self - a;
        let proj = ap.dot(ab);
        let d = proj / ab.length_squared();

        let p = if d <= 0.0 {
            a
        } else if d >= 1.0 {
            b
        } else {
            a + d * ab
        };

        self.distance(p)
    }
}
