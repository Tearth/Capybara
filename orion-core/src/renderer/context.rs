use super::{
    camera::Camera,
    shader::{Shader, *},
};
use crate::utils::storage::Storage;
use anyhow::Result;
use glam::{Vec2, Vec4};
use glow::{Context, HasContext};
use std::rc::Rc;

pub struct RendererContext {
    pub clear_color: Vec4,
    pub viewport_size: Vec2,

    pub active_camera_id: usize,
    pub active_shader_id: usize,

    pub cameras: Storage<Camera>,
    pub shaders: Storage<Shader>,
    pub gl: Rc<Context>,
}

impl RendererContext {
    pub fn new(gl: Context) -> Result<Self> {
        let mut context = Self {
            clear_color: Default::default(),
            viewport_size: Default::default(),

            active_camera_id: usize::MAX,
            active_shader_id: usize::MAX,

            cameras: Default::default(),
            shaders: Default::default(),
            gl: Rc::new(gl),
        };
        context.init()?;

        Ok(context)
    }

    fn init(&mut self) -> Result<()> {
        self.set_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

        let camera_id = self.cameras.store(Camera::new(Default::default(), Default::default()));
        self.activate_camera(camera_id)?;

        let shader_id = self.shaders.store(Shader::new(self, DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)?);
        self.activate_shader(shader_id)?;

        self.update_viewport()?;

        unsafe {
            let f32_size = core::mem::size_of::<f32>() as i32;
            let vertices = [
                0.0f32, 0.0f32, 0.0f32, 1.0, 0.0, 0.0, 1.0, 0.0f32, 0.0f32, /* 1 */
                300.0f32, 0.0f32, 0.0f32, 1.0, 0.0, 0.0, 1.0, 0.0f32, 1.0f32, /* 2 */
                150.0f32, 300.0f32, 0.0f32, 1.0, 0.0, 0.0, 1.0, 1.0f32, 1.0f32, /* 3 */
            ];
            let vertices_u8 = core::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * f32_size as usize);

            let vao = self.gl.create_vertex_array().unwrap();
            self.gl.bind_vertex_array(Some(vao));

            let vbo = self.gl.create_buffer().unwrap();
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            self.gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 9 * f32_size, 0);
            self.gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 9 * f32_size, 3 * f32_size);
            self.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 9 * f32_size, 7 * f32_size);

            self.gl.enable_vertex_attrib_array(0);
            self.gl.enable_vertex_attrib_array(1);
            self.gl.enable_vertex_attrib_array(2);
        }

        Ok(())
    }

    pub fn activate_camera(&mut self, camera_id: usize) -> Result<()> {
        let camera = self.cameras.get_mut(camera_id)?;
        camera.size = self.viewport_size;
        self.active_camera_id = camera_id;

        Ok(())
    }

    pub fn activate_shader(&mut self, shader_id: usize) -> Result<()> {
        let shader = self.shaders.get(shader_id)?;
        self.active_shader_id = shader_id;
        unsafe { self.gl.use_program(Some(shader.program)) };

        Ok(())
    }

    pub fn set_viewport(&mut self, size: Vec2) -> Result<()> {
        self.viewport_size = size;
        self.update_viewport()?;

        Ok(())
    }

    pub fn update_viewport(&mut self) -> Result<()> {
        let camera = self.cameras.get_mut(self.active_camera_id)?;
        camera.size = self.viewport_size;

        if self.active_shader_id != usize::MAX {
            let shader = self.shaders.get(self.active_shader_id)?;
            shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;
            shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr())?;
        }

        unsafe { self.gl.viewport(0, 0, self.viewport_size.x as i32, self.viewport_size.y as i32) };

        Ok(())
    }

    pub fn set_clear_color(&mut self, color: Vec4) {
        unsafe { self.gl.clear_color(color.x, color.y, color.z, color.w) };
        self.clear_color = color;
    }

    pub fn clear(&self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    pub fn get_version(&self) -> String {
        let version = self.gl.version();
        format!("{}.{} {}", version.major, version.minor, version.vendor_info)
    }
}
