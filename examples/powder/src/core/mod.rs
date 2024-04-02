use capybara::glam::Vec2;

pub mod persistence;
pub mod selector;

pub const PARTICLE_SIZE: i32 = 4;
pub const GRAVITY_DEFAULT: Vec2 = Vec2::new(0.0, -160.0);
pub const PIXELS_PER_METER: i32 = 50;
pub const CHUNK_SIZE: i32 = 32;
