use super::shader::{Shader, *};
use crate::utils::storage::Storage;
use anyhow::Result;
use glam::{Vec2, Vec4};
use glow::{Context, HasContext};
use std::rc::Rc;

pub struct RendererContext {
    pub clear_color: Vec4,

    pub default_shader_id: usize,
    pub active_shader_id: usize,

    pub shaders: Storage<Shader>,
    pub gl: Rc<Context>,
}

impl RendererContext {
    pub fn new(gl: Context) -> Result<Self> {
        let mut context =
            Self { clear_color: Default::default(), default_shader_id: usize::MAX, active_shader_id: usize::MAX, shaders: Default::default(), gl: Rc::new(gl) };
        context.init()?;

        Ok(context)
    }

    fn init(&mut self) -> Result<()> {
        self.set_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

        self.default_shader_id = self.shaders.store(Shader::new(self, DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)?);
        self.activate_shader(self.default_shader_id)?;

        Ok(())
    }

    pub fn activate_shader(&mut self, shader_id: usize) -> Result<()> {
        let shader = self.shaders.get(self.default_shader_id)?;
        self.active_shader_id = shader_id;

        unsafe { self.gl.use_program(Some(shader.program)) };
        Ok(())
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
