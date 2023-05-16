use super::camera::Camera;
use super::camera::CameraOrigin;
use super::shader::Shader;
use super::shader::*;
use super::sprite::Shape;
use super::sprite::Sprite;
use super::texture::Texture;
use crate::utils::storage::Storage;
use anyhow::Result;
use glam::Vec2;
use glam::Vec4;
use glow::Buffer;
use glow::Context;
use glow::HasContext;
use glow::VertexArray;
use instant::Instant;
use std::cmp;
use std::ptr;
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
    pub textures: Storage<Texture>,
    pub gl: Rc<Context>,

    buffer_vao: VertexArray,
    buffer_vbo: Buffer,
    buffer_ebo: Buffer,

    buffer_vertices_queue: Vec<f32>,
    buffer_indices_queue: Vec<u32>,
    buffer_vertices_count: usize,
    buffer_indices_count: usize,
    buffer_indices_max: u32,
    buffer_resized: bool,
    buffer_metadata: Option<BufferMetadata>,

    fps_timestamp: Instant,
    fps_count: u32,
    pub fps: u32,
}

#[derive(Debug, Default)]
pub struct BufferMetadata {
    pub texture_id: usize,
}

impl RendererContext {
    pub fn new(gl: Context) -> Result<Self> {
        unsafe {
            let square_vao = gl.create_vertex_array().unwrap();
            let square_vbo = gl.create_buffer().unwrap();
            let square_ebo = gl.create_buffer().unwrap();

            let mut context = Self {
                clear_color: Default::default(),
                viewport_size: Default::default(),

                default_camera_id: usize::MAX,
                active_camera_id: usize::MAX,
                default_shader_id: usize::MAX,
                active_shader_id: usize::MAX,

                cameras: Default::default(),
                shaders: Default::default(),
                textures: Default::default(),
                gl: Rc::new(gl),

                buffer_vao: square_vao,
                buffer_vbo: square_vbo,
                buffer_ebo: square_ebo,

                buffer_vertices_queue: vec![0.0; 256],
                buffer_indices_queue: vec![0; 256],
                buffer_vertices_count: 0,
                buffer_indices_count: 0,
                buffer_indices_max: 0,
                buffer_resized: true,
                buffer_metadata: None,

                fps_timestamp: Instant::now(),
                fps_count: 0,
                fps: 0,
            };
            context.init()?;

            Ok(context)
        }
    }

    fn init(&mut self) -> Result<()> {
        unsafe {
            self.gl.enable(glow::BLEND);
            self.gl.blend_equation_separate(glow::FUNC_ADD, glow::FUNC_ADD);
            self.gl.blend_func_separate(glow::ONE, glow::ONE_MINUS_SRC_ALPHA, glow::ONE_MINUS_DST_ALPHA, glow::ONE);
            self.set_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

            self.default_camera_id = self.cameras.store(Camera::new(Default::default(), Default::default(), CameraOrigin::LeftBottom));
            self.activate_camera(self.default_camera_id)?;

            self.default_shader_id = self.shaders.store(Shader::new(self, DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)?);
            self.activate_shader(self.default_shader_id)?;

            self.gl.bind_vertex_array(Some(self.buffer_vao));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer_vbo));
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffer_ebo));

            self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8 * 4, 0);
            self.gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 8 * 4, 2 * 4);
            self.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 8 * 4, 6 * 4);

            self.gl.enable_vertex_attrib_array(0);
            self.gl.enable_vertex_attrib_array(1);
            self.gl.enable_vertex_attrib_array(2);

            Ok(())
        }
    }

    pub fn begin_user_frame(&mut self) -> Result<()> {
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

    pub fn end_user_frame(&mut self) -> Result<()> {
        self.flush()?;

        let now = Instant::now();
        if (now - self.fps_timestamp).as_secs() >= 1 {
            self.fps = self.fps_count;
            self.fps_count = 0;
            self.fps_timestamp = now;
        } else {
            self.fps_count += 1;
        }

        Ok(())
    }

    pub fn draw(&mut self, sprite: &Sprite) -> Result<()> {
        let v_base = self.buffer_vertices_count;
        let i_base = self.buffer_indices_count;
        let mut flush = false;

        if let Some(metadata) = self.buffer_metadata.as_mut() {
            if metadata.texture_id != sprite.texture_id {
                flush = true;
            }
        }

        if flush {
            self.flush()?;
        }

        if self.buffer_metadata.is_none() {
            let mut metadata = BufferMetadata::default();
            metadata.texture_id = sprite.texture_id;

            self.buffer_metadata = Some(metadata);
        }

        match &sprite.shape {
            Shape::Standard => {
                if v_base + 32 >= self.buffer_vertices_queue.len() {
                    self.buffer_vertices_queue.resize(self.buffer_vertices_queue.len() * 2, 0.0);
                    self.buffer_resized = true;
                }

                if i_base + 6 >= self.buffer_indices_queue.len() {
                    self.buffer_indices_queue.resize(self.buffer_indices_queue.len() * 2, 0);
                    self.buffer_resized = true;
                }

                let model = sprite.get_model();
                let v1 = model * Vec4::new(0.0, 0.0, 0.0, 1.0);
                let v2 = model * Vec4::new(1.0, 0.0, 0.0, 1.0);
                let v3 = model * Vec4::new(1.0, 1.0, 0.0, 1.0);
                let v4 = model * Vec4::new(0.0, 1.0, 0.0, 1.0);

                self.buffer_vertices_queue[v_base + 0] = v1.x;
                self.buffer_vertices_queue[v_base + 1] = v1.y;
                self.buffer_vertices_queue[v_base + 2] = 1.0;
                self.buffer_vertices_queue[v_base + 3] = 1.0;
                self.buffer_vertices_queue[v_base + 4] = 1.0;
                self.buffer_vertices_queue[v_base + 5] = 1.0;
                self.buffer_vertices_queue[v_base + 6] = 0.0;
                self.buffer_vertices_queue[v_base + 7] = 1.0;

                self.buffer_vertices_queue[v_base + 8] = v2.x;
                self.buffer_vertices_queue[v_base + 9] = v2.y;
                self.buffer_vertices_queue[v_base + 10] = 1.0;
                self.buffer_vertices_queue[v_base + 11] = 1.0;
                self.buffer_vertices_queue[v_base + 12] = 1.0;
                self.buffer_vertices_queue[v_base + 13] = 1.0;
                self.buffer_vertices_queue[v_base + 14] = 1.0;
                self.buffer_vertices_queue[v_base + 15] = 1.0;

                self.buffer_vertices_queue[v_base + 16] = v3.x;
                self.buffer_vertices_queue[v_base + 17] = v3.y;
                self.buffer_vertices_queue[v_base + 18] = 1.0;
                self.buffer_vertices_queue[v_base + 19] = 1.0;
                self.buffer_vertices_queue[v_base + 20] = 1.0;
                self.buffer_vertices_queue[v_base + 21] = 1.0;
                self.buffer_vertices_queue[v_base + 22] = 1.0;
                self.buffer_vertices_queue[v_base + 23] = 0.0;

                self.buffer_vertices_queue[v_base + 24] = v4.x;
                self.buffer_vertices_queue[v_base + 25] = v4.y;
                self.buffer_vertices_queue[v_base + 26] = 1.0;
                self.buffer_vertices_queue[v_base + 27] = 1.0;
                self.buffer_vertices_queue[v_base + 28] = 1.0;
                self.buffer_vertices_queue[v_base + 29] = 1.0;
                self.buffer_vertices_queue[v_base + 30] = 0.0;
                self.buffer_vertices_queue[v_base + 31] = 0.0;

                self.buffer_indices_queue[i_base + 0] = self.buffer_indices_max + 0;
                self.buffer_indices_queue[i_base + 1] = self.buffer_indices_max + 1;
                self.buffer_indices_queue[i_base + 2] = self.buffer_indices_max + 2;
                self.buffer_indices_queue[i_base + 3] = self.buffer_indices_max + 0;
                self.buffer_indices_queue[i_base + 4] = self.buffer_indices_max + 2;
                self.buffer_indices_queue[i_base + 5] = self.buffer_indices_max + 3;

                self.buffer_vertices_count = v_base + 32;
                self.buffer_indices_count = i_base + 6;
                self.buffer_indices_max += 4;
            }
            Shape::Custom(data) => {
                loop {
                    let mut sufficient_space = true;

                    if v_base + data.vertices.len() >= self.buffer_vertices_queue.len() {
                        self.buffer_vertices_queue.resize(self.buffer_vertices_queue.len() * 2, 0.0);
                        self.buffer_resized = true;
                        sufficient_space = false;
                    }

                    if i_base + data.indices.len() >= self.buffer_indices_queue.len() {
                        self.buffer_indices_queue.resize(self.buffer_indices_queue.len() * 2, 0);
                        self.buffer_resized = true;
                        sufficient_space = false;
                    }

                    if sufficient_space {
                        break;
                    }
                }

                unsafe {
                    ptr::copy(data.vertices.as_ptr(), (self.buffer_vertices_queue.as_mut_ptr()).add(v_base), data.vertices.len());
                }

                let base_indice = self.buffer_indices_max;
                for i in 0..data.indices.len() {
                    self.buffer_indices_queue[i_base + i] = base_indice + data.indices[i];
                    self.buffer_indices_max = cmp::max(self.buffer_indices_max, base_indice + data.indices[i]);
                }

                self.buffer_vertices_count = v_base + data.vertices.len();
                self.buffer_indices_count = i_base + data.indices.len();
            }
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        unsafe {
            if let Some(metadata) = &self.buffer_metadata {
                if self.buffer_indices_count > 0 {
                    let mut camera = self.cameras.get_mut(self.active_camera_id)?;

                    if camera.dirty {
                        let shader = self.shaders.get(self.active_shader_id)?;
                        shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;
                        shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr())?;

                        camera.dirty = false;
                    }

                    if self.buffer_resized {
                        self.gl.buffer_data_size(glow::ARRAY_BUFFER, self.buffer_vertices_queue.len() as i32 * 4, glow::DYNAMIC_DRAW);
                        self.gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, self.buffer_indices_queue.len() as i32 * 4, glow::DYNAMIC_DRAW);
                        self.buffer_resized = false;
                    }

                    let models_u8 = core::slice::from_raw_parts(self.buffer_vertices_queue.as_ptr() as *const u8, self.buffer_vertices_count * 4 as usize);
                    let indices_u8 = core::slice::from_raw_parts(self.buffer_indices_queue.as_ptr() as *const u8, self.buffer_indices_count * 4 as usize);

                    self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, models_u8);
                    self.gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, 0, indices_u8);

                    self.textures.get(metadata.texture_id)?.activate();
                    self.gl.draw_elements(glow::TRIANGLES, self.buffer_indices_count as i32, glow::UNSIGNED_INT, 0);

                    self.buffer_vertices_count = 0;
                    self.buffer_indices_count = 0;
                    self.buffer_indices_max = 0;
                }

                self.buffer_metadata = None;
            }

            Ok(())
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
        self.active_shader_id = shader_id;

        self.shaders.get(shader_id)?.activate();
        self.cameras.get_mut(self.active_camera_id)?.dirty = true;

        Ok(())
    }

    pub fn set_viewport(&mut self, size: Vec2) -> Result<()> {
        unsafe {
            self.gl.viewport(0, 0, size.x as i32, size.y as i32);
            self.viewport_size = size;

            let camera = self.cameras.get_mut(self.active_camera_id)?;
            camera.size = self.viewport_size;
            camera.dirty = true;

            Ok(())
        }
    }

    pub fn set_clear_color(&mut self, color: Vec4) {
        unsafe {
            self.gl.clear_color(color.x, color.y, color.z, color.w);
            self.clear_color = color;
        }
    }

    pub fn enable_scissor(&self, position: Vec2, size: Vec2) {
        unsafe {
            self.gl.enable(glow::SCISSOR_TEST);
            self.gl.scissor(position.x as i32, position.y as i32, size.x as i32, size.y as i32);
        }
    }

    pub fn disable_scissor(&self) {
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
        }
    }
}
