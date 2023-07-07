use glam::{Vec2, Vec3, Vec4};

pub trait NewRand<T> {
    fn new_rand() -> T;
}

impl NewRand<f32> for f32 {
    fn new_rand() -> f32 {
        fastrand::f32()
    }
}

impl NewRand<Vec2> for Vec2 {
    fn new_rand() -> Vec2 {
        Vec2::new(fastrand::f32(), fastrand::f32())
    }
}

impl NewRand<Vec3> for Vec3 {
    fn new_rand() -> Vec3 {
        Vec3::new(fastrand::f32(), fastrand::f32(), fastrand::f32())
    }
}

impl NewRand<Vec4> for Vec4 {
    fn new_rand() -> Vec4 {
        Vec4::new(fastrand::f32(), fastrand::f32(), fastrand::f32(), fastrand::f32())
    }
}
