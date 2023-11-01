use colors_transform::Color;
use colors_transform::Rgb;
use glam::Vec4;

pub trait Vec4Color {
    fn new_rgb(r: u8, g: u8, b: u8, a: u8) -> Vec4;
    fn to_rgb(self) -> (u8, u8, u8, u8);
    fn to_rgb_packed(self) -> u32;
}

pub trait RgbToVec4 {
    fn to_vec4(self) -> Vec4;
}

impl Vec4Color for Vec4 {
    fn new_rgb(r: u8, g: u8, b: u8, a: u8) -> Vec4 {
        Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    fn to_rgb(self) -> (u8, u8, u8, u8) {
        ((self.x * self.w * 255.0) as u8, (self.y * self.w * 255.0) as u8, (self.z * self.w * 255.0) as u8, (self.w * 255.0) as u8)
    }

    fn to_rgb_packed(self) -> u32 {
        let (r, g, b, a) = self.to_rgb();
        r as u32 | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24)
    }
}

impl RgbToVec4 for Rgb {
    fn to_vec4(self) -> Vec4 {
        let (r, g, b) = self.as_tuple();
        Vec4::new(r / 255.0, g / 255.0, b / 255.0, 1.0)
    }
}
