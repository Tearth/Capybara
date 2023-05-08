use super::camera::Camera;
use super::camera::CameraOrigin;
use super::shader::Shader;
use super::shader::*;
use crate::utils::storage::Storage;
use anyhow::Result;
use glam::Vec2;
use glam::Vec4;
use glow::Context;
use glow::HasContext;
use instant::Instant;
use std::rc::Rc;

pub struct RendererContext {
    pub clear_color: Vec4,
    pub viewport_size: Vec2,

    pub default_camera_id: usize,
    pub active_camera_id: usize,
    pub default_shader_id: usize,
    pub active_shader_id: usize,

    pub cameras: Storage<Camera>,
    pub shaders: Storage<Shader>,
    pub gl: Rc<Context>,

    fps_timestamp: Instant,
    fps_count: u32,
    pub fps: u32,
}

impl RendererContext {
    pub fn new(gl: Context) -> Result<Self> {
        let mut context = Self {
            clear_color: Default::default(),
            viewport_size: Default::default(),

            default_camera_id: usize::MAX,
            active_camera_id: usize::MAX,
            default_shader_id: usize::MAX,
            active_shader_id: usize::MAX,

            cameras: Default::default(),
            shaders: Default::default(),
            gl: Rc::new(gl),

            fps_timestamp: Instant::now(),
            fps_count: 0,
            fps: 0,
        };
        context.init()?;

        Ok(context)
    }

    fn init(&mut self) -> Result<()> {
        unsafe {
            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.set_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

            self.default_camera_id = self.cameras.store(Camera::new(Default::default(), Default::default(), CameraOrigin::LeftTop));
            self.activate_camera(self.default_camera_id)?;

            self.default_shader_id = self.shaders.store(Shader::new(self, DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)?);
            self.activate_shader(self.default_shader_id)?;

            Ok(())
        }
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            if self.active_camera_id != self.default_camera_id {
                self.activate_camera(self.default_camera_id)?;
            }

            if self.active_shader_id != self.default_shader_id {
                self.activate_shader(self.default_shader_id)?;
            }

            Ok(())
        }
    }

    pub fn end_frame(&mut self) {
        let now = Instant::now();
        if (now - self.fps_timestamp).as_secs() >= 1 {
            self.fps = self.fps_count;
            self.fps_count = 0;
            self.fps_timestamp = now;
        } else {
            self.fps_count += 1;
        }
    }

    pub fn activate_camera(&mut self, camera_id: usize) -> Result<()> {
        let camera = self.cameras.get_mut(camera_id)?;
        self.active_camera_id = camera_id;

        camera.size = self.viewport_size;
        camera.dirty = true;

        Ok(())
    }

    pub fn activate_shader(&mut self, shader_id: usize) -> Result<()> {
        unsafe {
            let shader = self.shaders.get(shader_id)?;
            self.active_shader_id = shader_id;
            self.gl.use_program(Some(shader.program));

            let mut camera = self.cameras.get_mut(self.active_camera_id)?;
            shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;
            shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr())?;

            camera.dirty = false;

            Ok(())
        }
    }

    pub fn set_viewport(&mut self, size: Vec2) -> Result<()> {
        unsafe {
            self.gl.viewport(0, 0, size.x as i32, size.y as i32);
            self.viewport_size = size;

            let camera = self.cameras.get_mut(self.active_camera_id)?;
            camera.size = self.viewport_size;

            let shader = self.shaders.get(self.active_shader_id)?;
            shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;

            Ok(())
        }
    }

    pub fn set_clear_color(&mut self, color: Vec4) {
        unsafe {
            self.gl.clear_color(color.x, color.y, color.z, color.w);
            self.clear_color = color;
        }
    }
}
