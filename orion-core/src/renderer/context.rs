use super::camera::Camera;
use super::camera::CameraOrigin;
use super::shader::Shader;
use super::shader::*;
use super::sprite::Shape;
use super::sprite::Sprite;
use super::sprite::Tile;
use super::texture::AtlasEntity;
use super::texture::Texture;
use super::texture::TextureKind;
use crate::assets::loader::AssetsLoader;
use crate::utils::storage::Storage;
use anyhow::bail;
use anyhow::Result;
use glam::Vec2;
use glam::Vec4;
use glow::Buffer;
use glow::Context;
use glow::HasContext;
use glow::VertexArray;
use instant::Instant;
use rustc_hash::FxHashMap;
use std::cmp;
use std::path::Path;
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

    buffer_texture: Option<usize>,
    buffer_vertices_queue: Vec<f32>,
    buffer_indices_queue: Vec<u32>,
    buffer_vertices_count: usize,
    buffer_indices_count: usize,
    buffer_indices_max: u32,
    buffer_resized: bool,

    pub fps: u32,
    fps_timestamp: Instant,
    fps_count: u32,
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

                buffer_texture: None,
                buffer_vertices_queue: vec![0.0; 256],
                buffer_indices_queue: vec![0; 256],
                buffer_vertices_count: 0,
                buffer_indices_count: 0,
                buffer_indices_max: 0,
                buffer_resized: true,

                fps: 0,
                fps_timestamp: Instant::now(),
                fps_count: 0,
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

            let camera = Camera::new(Default::default(), Default::default(), CameraOrigin::LeftBottom);
            self.default_camera_id = self.cameras.store(camera);
            self.activate_camera(self.default_camera_id)?;

            let shader = Shader::new(self, DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)?;
            self.default_shader_id = self.shaders.store(shader);
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

    pub fn instantiate_assets(&mut self, assets: &AssetsLoader, prefix: Option<&str>) -> Result<()> {
        for texture in &assets.raw_textures {
            if let Some(prefix) = &prefix {
                if !texture.path.starts_with(prefix) {
                    continue;
                }
            }

            self.textures.store_with_name(&texture.name, Texture::new(self, texture))?;
        }

        for atlas in &assets.raw_atlases {
            let name_without_extension = Path::new(&atlas.name).file_stem().unwrap().to_str().unwrap();
            if self.textures.contains_by_name(name_without_extension) {
                let texture = self.textures.get_by_name_mut(name_without_extension)?;
                let mut entities = FxHashMap::default();

                for raw in &atlas.entities {
                    entities.insert(raw.name.clone(), AtlasEntity::new(raw.position, raw.size));
                }

                texture.kind = TextureKind::Atlas(entities);
            }
        }

        Ok(())
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
        let mut v_base = self.buffer_vertices_count;
        let mut i_base = self.buffer_indices_count;
        let mut flush = false;

        if let Some(texture_id) = self.buffer_texture.as_mut() {
            if *texture_id != sprite.texture_id {
                flush = true;
            }
        }

        if flush {
            self.flush()?;
            v_base = 0;
            i_base = 0;
        }

        if self.buffer_texture.is_none() {
            self.buffer_texture = Some(sprite.texture_id);
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

                let (uv_position, uv_size) = match &sprite.tile {
                    Tile::Simple => (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    Tile::Tilemap { size } => {
                        let texture = self.textures.get(sprite.texture_id)?;
                        let tiles_count = texture.size / *size;
                        let position = Vec2::new(
                            (sprite.animation_frame % tiles_count.x as usize) as f32,
                            (sprite.animation_frame / tiles_count.x as usize) as f32,
                        );
                        let uv_position = position / tiles_count;
                        let uv_size = *size / texture.size;

                        (uv_position, uv_size)
                    }
                    Tile::TilemapAnimation { size, frames } => {
                        let texture = self.textures.get(sprite.texture_id)?;
                        let tiles_count = texture.size / *size;
                        let frame = frames[sprite.animation_frame];
                        let position = Vec2::new((frame % tiles_count.x as usize) as f32, (frame / tiles_count.x as usize) as f32);
                        let uv_position = position / tiles_count;
                        let uv_size = *size / texture.size;

                        (uv_position, uv_size)
                    }
                    Tile::AtlasEntity { name } => {
                        let texture = self.textures.get(sprite.texture_id)?;
                        if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                            let entity = atlas_entities.get(name).unwrap();
                            (entity.position / texture.size, entity.size / texture.size)
                        } else {
                            bail!("Texture is not an atlas");
                        }
                    }
                    Tile::AtlasAnimation { entities } => {
                        let texture = self.textures.get(sprite.texture_id)?;
                        if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                            let name = &entities[sprite.animation_frame];
                            let entity = atlas_entities.get(name).unwrap();
                            (entity.position / texture.size, entity.size / texture.size)
                        } else {
                            bail!("Texture is not an atlas");
                        }
                    }
                };

                self.buffer_vertices_queue[v_base + 0] = v1.x;
                self.buffer_vertices_queue[v_base + 1] = v1.y;
                self.buffer_vertices_queue[v_base + 2] = sprite.color.x;
                self.buffer_vertices_queue[v_base + 3] = sprite.color.y;
                self.buffer_vertices_queue[v_base + 4] = sprite.color.z;
                self.buffer_vertices_queue[v_base + 5] = sprite.color.w;
                self.buffer_vertices_queue[v_base + 6] = uv_position.x;
                self.buffer_vertices_queue[v_base + 7] = uv_position.y + uv_size.y;

                self.buffer_vertices_queue[v_base + 8] = v2.x;
                self.buffer_vertices_queue[v_base + 9] = v2.y;
                self.buffer_vertices_queue[v_base + 10] = sprite.color.x;
                self.buffer_vertices_queue[v_base + 11] = sprite.color.y;
                self.buffer_vertices_queue[v_base + 12] = sprite.color.z;
                self.buffer_vertices_queue[v_base + 13] = sprite.color.w;
                self.buffer_vertices_queue[v_base + 14] = uv_position.x + uv_size.x;
                self.buffer_vertices_queue[v_base + 15] = uv_position.y + uv_size.y;

                self.buffer_vertices_queue[v_base + 16] = v3.x;
                self.buffer_vertices_queue[v_base + 17] = v3.y;
                self.buffer_vertices_queue[v_base + 18] = sprite.color.x;
                self.buffer_vertices_queue[v_base + 19] = sprite.color.y;
                self.buffer_vertices_queue[v_base + 20] = sprite.color.z;
                self.buffer_vertices_queue[v_base + 21] = sprite.color.w;
                self.buffer_vertices_queue[v_base + 22] = uv_position.x + uv_size.x;
                self.buffer_vertices_queue[v_base + 23] = uv_position.y;

                self.buffer_vertices_queue[v_base + 24] = v4.x;
                self.buffer_vertices_queue[v_base + 25] = v4.y;
                self.buffer_vertices_queue[v_base + 26] = sprite.color.x;
                self.buffer_vertices_queue[v_base + 27] = sprite.color.y;
                self.buffer_vertices_queue[v_base + 28] = sprite.color.z;
                self.buffer_vertices_queue[v_base + 29] = sprite.color.w;
                self.buffer_vertices_queue[v_base + 30] = uv_position.x;
                self.buffer_vertices_queue[v_base + 31] = uv_position.y;

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
            if let Some(texture_id) = &self.buffer_texture {
                if self.buffer_indices_count > 0 {
                    let mut camera = self.cameras.get_mut(self.active_camera_id)?;

                    if camera.dirty {
                        let shader = self.shaders.get(self.active_shader_id)?;
                        shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;
                        shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr())?;

                        camera.dirty = false;
                    }

                    if self.buffer_resized {
                        let buffer_vertices_size = self.buffer_vertices_queue.len() as i32 * 4;
                        let buffer_indices_size = self.buffer_indices_queue.len() as i32 * 4;

                        self.gl.buffer_data_size(glow::ARRAY_BUFFER, buffer_vertices_size, glow::DYNAMIC_DRAW);
                        self.gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, buffer_indices_size, glow::DYNAMIC_DRAW);

                        self.buffer_resized = false;
                    }

                    let models_u8 = core::slice::from_raw_parts(self.buffer_vertices_queue.as_ptr() as *const u8, self.buffer_vertices_count * 4);
                    let indices_u8 = core::slice::from_raw_parts(self.buffer_indices_queue.as_ptr() as *const u8, self.buffer_indices_count * 4);

                    self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, models_u8);
                    self.gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, 0, indices_u8);

                    self.textures.get(*texture_id)?.activate();
                    self.gl.draw_elements(glow::TRIANGLES, self.buffer_indices_count as i32, glow::UNSIGNED_INT, 0);

                    self.buffer_vertices_count = 0;
                    self.buffer_indices_count = 0;
                    self.buffer_indices_max = 0;
                }

                self.buffer_texture = None;
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
