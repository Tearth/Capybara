use glam::Vec4;

pub trait Vec4Color {
    fn new_rgb(r: u8, g: u8, b: u8, a: u8) -> Vec4;
}

impl Vec4Color for Vec4 {
    fn new_rgb(r: u8, g: u8, b: u8, a: u8) -> Vec4 {
        Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }
}
