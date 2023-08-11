use glam::Vec2;
use glam::Vec3;
use glam::Vec4;
use std::ops::Bound;
use std::ops::RangeBounds;

pub trait NewRand<T> {
    fn new_rand(range: impl RangeBounds<f32> + Clone) -> T;
}

impl NewRand<f32> for f32 {
    fn new_rand(range: impl RangeBounds<f32> + Clone) -> f32 {
        let from = match range.start_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => f32::MIN,
        };

        let to = match range.end_bound() {
            Bound::Included(v) | Bound::Excluded(v) => *v,
            Bound::Unbounded => f32::MAX,
        };

        fastrand::f32() * (to - from) + from
    }
}

impl NewRand<Vec2> for Vec2 {
    fn new_rand(range: impl RangeBounds<f32> + Clone) -> Vec2 {
        Vec2::new(f32::new_rand(range.clone()), f32::new_rand(range.clone()))
    }
}

impl NewRand<Vec3> for Vec3 {
    fn new_rand(range: impl RangeBounds<f32> + Clone) -> Vec3 {
        Vec3::new(f32::new_rand(range.clone()), f32::new_rand(range.clone()), f32::new_rand(range.clone()))
    }
}

impl NewRand<Vec4> for Vec4 {
    fn new_rand(range: impl RangeBounds<f32> + Clone) -> Vec4 {
        Vec4::new(f32::new_rand(range.clone()), f32::new_rand(range.clone()), f32::new_rand(range.clone()), f32::new_rand(range.clone()))
    }
}
