use std::rc::Rc;

use glam::{Vec2, Vec4};
use glow::{Context, HasContext};

pub struct RendererContext {
    pub clear_color: Vec4,

    pub gl: Rc<Context>,
}

impl RendererContext {
    pub fn new(gl: Context) -> Self {
        let mut context = Self { clear_color: Default::default(), gl: Rc::new(gl) };
        context.set_clear_color(Vec4::new(0.2, 0.2, 0.2, 0.2));

        context
    }

    pub fn set_viewport(&mut self, size: Vec2) {
        unsafe {
            self.gl.viewport(0, 0, size.x as i32, size.y as i32);
        }
    }

    pub fn set_clear_color(&mut self, color: Vec4) {
        unsafe {
            self.gl.clear_color(color.x, color.y, color.z, color.w);
        }
        self.clear_color = color;
    }

    pub fn clear(&self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn get_version(&self) -> String {
        let version = self.gl.version();
        format!("{}.{} {}", version.major, version.minor, version.vendor_info)
    }
}
