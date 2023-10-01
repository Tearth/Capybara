use std::f32::consts;

pub trait NormalizeAngle {
    fn normalize_angle(&self) -> f32;
}

impl NormalizeAngle for f32 {
    fn normalize_angle(&self) -> f32 {
        let angle = (self + consts::TAU) % consts::TAU;

        if angle > consts::PI {
            angle - consts::TAU
        } else {
            angle
        }
    }
}
