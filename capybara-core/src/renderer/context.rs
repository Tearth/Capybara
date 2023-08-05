use super::camera::Camera;
use super::camera::CameraOrigin;
use super::shader::Shader;
use super::shader::*;
use super::shape::Shape;
use super::sprite::Sprite;
use super::sprite::TextureType;
use super::texture::AtlasEntity;
use super::texture::Texture;
use super::texture::TextureKind;
use crate::assets::loader::AssetsLoader;
use crate::assets::RawTexture;
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
use std::slice;

pub struct RendererContext {
    pub viewport_size: Vec2,

    pub default_camera_id: usize,
    pub default_sprite_shader_id: usize,
    pub default_shape_shader_id: usize,
    pub default_texture_id: usize,

    pub active_camera_id: usize,
    pub active_shader_id: usize,

    pub cameras: Storage<Camera>,
    pub shaders: Storage<Shader>,
    pub textures: Storage<Texture>,
    pub gl: Rc<Context>,

    buffer_metadata: Option<BufferMetadata>,

    sprite_buffer_vao: VertexArray,
    sprite_buffer_base_vbo: Buffer,
    sprite_buffer_data_vbo: Buffer,
    sprite_buffer_ebo: Buffer,
    sprite_buffer_resized: bool,
    sprite_buffer_count: usize,
    sprite_buffer_vertices_queue: Vec<u32>,
    sprite_buffer_vertices_count: usize,

    shape_buffer_vao: VertexArray,
    shape_buffer_vbo: Buffer,
    shape_buffer_ebo: Buffer,
    shape_buffer_resized: bool,
    shape_buffer_vertices_queue: Vec<u32>,
    shape_buffer_indices_queue: Vec<u32>,
    shape_buffer_vertices_count: usize,
    shape_buffer_indices_count: usize,
    shape_buffer_indices_max: u32,

    pub fps: u32,
    fps_timestamp: Instant,
    fps_count: u32,
}

pub struct BufferMetadata {
    pub content_type: BufferContentType,
    pub texture_id: Option<usize>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BufferContentType {
    Sprite,
    Shape,
}

impl RendererContext {
    pub fn new(gl: Context) -> Result<Self> {
        unsafe {
            let sprite_buffer_vao = gl.create_vertex_array().unwrap();
            let sprite_buffer_base_vbo = gl.create_buffer().unwrap();
            let sprite_buffer_data_vbo = gl.create_buffer().unwrap();
            let sprite_buffer_ebo = gl.create_buffer().unwrap();

            let shape_buffer_vao = gl.create_vertex_array().unwrap();
            let shape_buffer_vbo = gl.create_buffer().unwrap();
            let shape_buffer_ebo = gl.create_buffer().unwrap();

            let mut context = Self {
                viewport_size: Default::default(),

                default_camera_id: usize::MAX,
                default_sprite_shader_id: usize::MAX,
                default_shape_shader_id: usize::MAX,
                default_texture_id: usize::MAX,

                active_camera_id: usize::MAX,
                active_shader_id: usize::MAX,

                cameras: Default::default(),
                shaders: Default::default(),
                textures: Default::default(),
                gl: Rc::new(gl),

                buffer_metadata: None,

                sprite_buffer_vao,
                sprite_buffer_base_vbo,
                sprite_buffer_data_vbo,
                sprite_buffer_ebo,
                sprite_buffer_resized: true,
                sprite_buffer_count: 0,
                sprite_buffer_vertices_queue: vec![0; 256],
                sprite_buffer_vertices_count: 0,

                shape_buffer_vao,
                shape_buffer_vbo,
                shape_buffer_ebo,
                shape_buffer_resized: true,
                shape_buffer_vertices_queue: vec![0; 256],
                shape_buffer_indices_queue: vec![0; 256],
                shape_buffer_vertices_count: 0,
                shape_buffer_indices_count: 0,
                shape_buffer_indices_max: 0,

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
            self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.set_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

            let camera = Camera::new(Default::default(), Default::default(), CameraOrigin::LeftBottom);
            self.default_camera_id = self.cameras.store(camera);
            self.activate_camera(self.default_camera_id)?;

            let sprite_shader = Shader::new(self, SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER)?;
            self.default_sprite_shader_id = self.shaders.store(sprite_shader);

            let shape_shader = Shader::new(self, SHAPE_VERTEX_SHADER, SHAPE_FRAGMENT_SHADER)?;
            self.default_shape_shader_id = self.shaders.store(shape_shader);

            let default_texture = Texture::new(self, &RawTexture::new("", "", Vec2::new(1.0, 1.0), &[255, 255, 255, 255]));
            self.default_texture_id = self.textures.store(default_texture);

            // Sprite buffers
            self.gl.bind_vertex_array(Some(self.sprite_buffer_vao));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.sprite_buffer_base_vbo));

            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.sprite_buffer_ebo));
            self.gl.enable_vertex_attrib_array(0);
            self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 2 * 4, 0);

            let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
            let indices = [0, 1, 2, 0, 2, 3];

            let models_u8 = core::slice::from_raw_parts(vertices.as_ptr() as *const u8, 8 * 4);
            let indices_u8 = core::slice::from_raw_parts(indices.as_ptr() as *const u8, 6 * 4);

            self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, models_u8, glow::STATIC_DRAW);
            self.gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STATIC_DRAW);

            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.sprite_buffer_data_vbo));

            self.gl.enable_vertex_attrib_array(1);
            self.gl.enable_vertex_attrib_array(2);
            self.gl.enable_vertex_attrib_array(3);
            self.gl.enable_vertex_attrib_array(4);
            self.gl.enable_vertex_attrib_array(5);
            self.gl.enable_vertex_attrib_array(6);

            self.gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 12 * 4, 0);
            self.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 12 * 4, 2 * 4);
            self.gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, 12 * 4, 4 * 4);
            self.gl.vertex_attrib_pointer_f32(4, 2, glow::FLOAT, false, 12 * 4, 5 * 4);
            self.gl.vertex_attrib_pointer_i32(5, 4, glow::UNSIGNED_BYTE, 12 * 4, 7 * 4);
            self.gl.vertex_attrib_pointer_f32(6, 4, glow::FLOAT, false, 12 * 4, 8 * 4);

            self.gl.vertex_attrib_divisor(1, 1);
            self.gl.vertex_attrib_divisor(2, 1);
            self.gl.vertex_attrib_divisor(3, 1);
            self.gl.vertex_attrib_divisor(4, 1);
            self.gl.vertex_attrib_divisor(5, 1);
            self.gl.vertex_attrib_divisor(6, 1);

            // UI buffers
            self.gl.bind_vertex_array(Some(self.shape_buffer_vao));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.shape_buffer_vbo));
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.shape_buffer_ebo));

            self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 5 * 4, 0);
            self.gl.vertex_attrib_pointer_i32(1, 4, glow::UNSIGNED_BYTE, 5 * 4, 2 * 4);
            self.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 5 * 4, 3 * 4);

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

    pub fn begin_frame(&mut self) -> Result<()> {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            if self.active_camera_id != self.default_camera_id {
                self.activate_camera(self.default_camera_id)?;
            }

            Ok(())
        }
    }

    pub fn end_frame(&mut self) -> Result<()> {
        self.flush_buffer()?;

        unsafe {
            self.gl.flush();
        }

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

    pub fn draw_sprite(&mut self, sprite: &Sprite) -> Result<()> {
        let camera = self.cameras.get(self.active_camera_id)?;
        let sprite_size = if let Some(texture_id) = sprite.texture_id {
            let texture = self.textures.get(texture_id)?;

            match &sprite.texture_type {
                TextureType::Simple => sprite.size.unwrap_or(texture.size),
                TextureType::SimpleOffset { offset: _ } => sprite.size.unwrap_or(texture.size),
                TextureType::Tilemap { size } => sprite.size.unwrap_or(*size),
                TextureType::TilemapAnimation { size, frames: _ } => sprite.size.unwrap_or(*size),
                TextureType::AtlasEntity { name } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let entity = atlas_entities.get(name).unwrap();
                        sprite.size.unwrap_or(entity.size)
                    } else {
                        bail!("Texture is not an atlas");
                    }
                }
                TextureType::AtlasAnimation { entities } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let name = &entities[sprite.animation_frame];
                        let entity = atlas_entities.get(name).unwrap();
                        sprite.size.unwrap_or(entity.size)
                    } else {
                        bail!("Texture is not an atlas");
                    }
                }
            }
        } else {
            sprite.size.unwrap_or(Vec2::new(1.0, 1.0))
        };

        let camera_min = camera.position;
        let camera_max = camera.position + camera.size;
        let sprite_min = sprite.position - sprite.anchor * sprite_size;
        let sprite_max = sprite.position + (Vec2::new(1.0, 1.0) - sprite.anchor) * sprite_size;

        if sprite_min.x > camera_max.x || sprite_min.y > camera_max.y || sprite_max.x < camera_min.x || sprite_max.y < camera_min.y {
            return Ok(());
        }

        if let Some(buffer_metadata) = &self.buffer_metadata {
            if buffer_metadata.content_type != BufferContentType::Sprite || buffer_metadata.texture_id != sprite.texture_id {
                self.flush_buffer()?;
                self.buffer_metadata = Some(BufferMetadata::new(BufferContentType::Sprite, sprite.texture_id));
            }
        } else {
            self.buffer_metadata = Some(BufferMetadata::new(BufferContentType::Sprite, sprite.texture_id));
        }

        if self.sprite_buffer_vertices_count + 12 >= self.sprite_buffer_vertices_queue.len() {
            self.sprite_buffer_vertices_queue.resize(self.sprite_buffer_vertices_queue.len() * 2, 0);
            self.sprite_buffer_resized = true;
        }

        let (uv_position, uv_size) = if let Some(texture_id) = sprite.texture_id {
            let texture = self.textures.get(texture_id)?;

            match &sprite.texture_type {
                TextureType::Simple => (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                TextureType::SimpleOffset { offset } => {
                    let uv_position = *offset / texture.size;
                    let uv_size = sprite_size / texture.size;

                    (uv_position, uv_size)
                }
                TextureType::Tilemap { size } => {
                    let tiles_count = texture.size / *size;
                    let tile_x = sprite.animation_frame % tiles_count.x as usize;
                    let tile_y = sprite.animation_frame / tiles_count.x as usize;
                    let position = Vec2::new(tile_x as f32, tile_y as f32);
                    let uv_position = position / tiles_count;
                    let uv_size = *size / texture.size;

                    (uv_position, uv_size)
                }
                TextureType::TilemapAnimation { size, frames } => {
                    let tiles_count = texture.size / *size;
                    let frame = frames[sprite.animation_frame];
                    let frame_x = frame % tiles_count.x as usize;
                    let frame_y = frame / tiles_count.x as usize;
                    let position = Vec2::new(frame_x as f32, frame_y as f32);
                    let uv_position = position / tiles_count;
                    let uv_size = *size / texture.size;

                    (uv_position, uv_size)
                }
                TextureType::AtlasEntity { name } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let entity = atlas_entities.get(name).unwrap();
                        (entity.position / texture.size, entity.size / texture.size)
                    } else {
                        bail!("Texture is not an atlas");
                    }
                }
                TextureType::AtlasAnimation { entities } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let name = &entities[sprite.animation_frame];
                        let entity = atlas_entities.get(name).unwrap();

                        (entity.position / texture.size, entity.size / texture.size)
                    } else {
                        bail!("Texture is not an atlas");
                    }
                }
            }
        } else {
            (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))
        };

        let r = (sprite.color.x * sprite.color.w * 255.0) as u32;
        let g = (sprite.color.y * sprite.color.w * 255.0) as u32;
        let b = (sprite.color.z * sprite.color.w * 255.0) as u32;
        let a = (sprite.color.w * 255.0) as u32;
        let color = r | (g << 8) | (b << 16) | (a << 24);

        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 0] = sprite.position.x.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 1] = sprite.position.y.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 2] = sprite.anchor.x.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 3] = sprite.anchor.y.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 4] = sprite.rotation.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 5] = (sprite_size.x * sprite.scale.x).to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 6] = (sprite_size.y * sprite.scale.y).to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 7] = color;
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 8] = uv_position.x.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 9] = uv_position.y.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 10] = uv_size.x.to_bits();
        self.sprite_buffer_vertices_queue[self.sprite_buffer_vertices_count + 11] = uv_size.y.to_bits();

        self.sprite_buffer_count += 1;
        self.sprite_buffer_vertices_count += 12;

        Ok(())
    }

    pub fn draw_shape(&mut self, shape: &Shape) -> Result<()> {
        if let Some(buffer_metadata) = &self.buffer_metadata {
            if buffer_metadata.content_type != BufferContentType::Shape || buffer_metadata.texture_id != shape.texture_id {
                self.flush_buffer()?;
                self.buffer_metadata = Some(BufferMetadata::new(BufferContentType::Shape, shape.texture_id));
            }
        } else {
            self.buffer_metadata = Some(BufferMetadata::new(BufferContentType::Shape, shape.texture_id));
        }

        loop {
            let mut sufficient_space = true;

            if self.shape_buffer_vertices_count + shape.vertices.len() >= self.shape_buffer_vertices_queue.len() {
                self.shape_buffer_vertices_queue.resize(self.shape_buffer_vertices_queue.len() * 2, 0);
                self.shape_buffer_resized = true;
                sufficient_space = false;
            }

            if self.shape_buffer_indices_count + shape.indices.len() >= self.shape_buffer_indices_queue.len() {
                self.shape_buffer_indices_queue.resize(self.shape_buffer_indices_queue.len() * 2, 0);
                self.shape_buffer_resized = true;
                sufficient_space = false;
            }

            if sufficient_space {
                break;
            }
        }

        unsafe {
            let buffer_ptr = self.shape_buffer_vertices_queue.as_mut_ptr();
            ptr::copy(shape.vertices.as_ptr(), buffer_ptr.add(self.shape_buffer_vertices_count), shape.vertices.len());
        }

        if shape.apply_model {
            let model = shape.get_model();

            for i in 0..self.shape_buffer_vertices_queue.len() / 5 {
                let x = self.shape_buffer_vertices_queue[i * 5 + 0];
                let y = self.shape_buffer_vertices_queue[i * 5 + 1];
                let position = Vec4::new(f32::from_bits(x), f32::from_bits(y), 0.0, 1.0);
                let position_transformed = model * position;

                self.shape_buffer_vertices_queue[i * 5 + 0] = position_transformed.x.to_bits();
                self.shape_buffer_vertices_queue[i * 5 + 1] = position_transformed.y.to_bits();
            }
        }

        let base_indice = self.shape_buffer_indices_max;
        for i in 0..shape.indices.len() {
            self.shape_buffer_indices_queue[self.shape_buffer_indices_count + i] = base_indice + shape.indices[i];
            self.shape_buffer_indices_max = cmp::max(self.shape_buffer_indices_max, base_indice + shape.indices[i]);
        }

        self.shape_buffer_vertices_count += shape.vertices.len();
        self.shape_buffer_indices_count += shape.indices.len();

        Ok(())
    }

    pub fn flush_buffer(&mut self) -> Result<()> {
        unsafe {
            if let Some(buffer_metadata) = &self.buffer_metadata {
                let camera = self.cameras.get_mut(self.active_camera_id)?;

                if self.sprite_buffer_resized {
                    let buffer_vertices_size = self.sprite_buffer_vertices_queue.len() as i32 * 4;

                    self.gl.bind_vertex_array(Some(self.sprite_buffer_vao));
                    self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.sprite_buffer_data_vbo));
                    self.gl.buffer_data_size(glow::ARRAY_BUFFER, buffer_vertices_size, glow::DYNAMIC_DRAW);

                    self.sprite_buffer_resized = false;
                }

                if self.shape_buffer_resized {
                    let buffer_vertices_size = self.shape_buffer_vertices_queue.len() as i32 * 4;
                    let buffer_indices_size = self.shape_buffer_indices_queue.len() as i32 * 4;

                    self.gl.bind_vertex_array(Some(self.shape_buffer_vao));
                    self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.shape_buffer_vbo));
                    self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.shape_buffer_ebo));
                    self.gl.buffer_data_size(glow::ARRAY_BUFFER, buffer_vertices_size, glow::DYNAMIC_DRAW);
                    self.gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, buffer_indices_size, glow::DYNAMIC_DRAW);

                    self.shape_buffer_resized = false;
                }

                match buffer_metadata.content_type {
                    BufferContentType::Sprite => {
                        if self.active_shader_id != self.default_sprite_shader_id || camera.dirty {
                            let shader = self.shaders.get(self.default_sprite_shader_id)?;
                            shader.activate();
                            shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;
                            shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr())?;

                            self.active_shader_id = self.default_sprite_shader_id;
                            camera.dirty = false;
                        }

                        self.gl.bind_vertex_array(Some(self.sprite_buffer_vao));
                        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.sprite_buffer_data_vbo));
                        self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.sprite_buffer_ebo));

                        let models_u8 = core::slice::from_raw_parts(
                            self.sprite_buffer_vertices_queue.as_ptr() as *const u8,
                            self.sprite_buffer_vertices_count * 4,
                        );

                        self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, models_u8);

                        if let Some(texture_id) = buffer_metadata.texture_id {
                            self.textures.get(texture_id)?.activate();
                        } else {
                            self.textures.get(self.default_texture_id)?.activate();
                        }

                        self.gl.draw_elements_instanced(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0, self.sprite_buffer_count as i32);

                        self.sprite_buffer_count = 0;
                        self.sprite_buffer_vertices_count = 0;
                    }
                    BufferContentType::Shape => {
                        if self.active_shader_id != self.default_shape_shader_id || camera.dirty {
                            let shader = self.shaders.get(self.default_shape_shader_id)?;
                            shader.activate();
                            shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr())?;
                            shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr())?;

                            self.active_shader_id = self.default_shape_shader_id;
                            camera.dirty = false;
                        }

                        self.gl.bind_vertex_array(Some(self.shape_buffer_vao));
                        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.shape_buffer_vbo));
                        self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.shape_buffer_ebo));

                        let buffer_ptr = self.shape_buffer_vertices_queue.as_ptr();
                        let models_u8 = slice::from_raw_parts(buffer_ptr as *const u8, self.shape_buffer_vertices_count * 4);

                        let buffer_ptr = self.shape_buffer_indices_queue.as_ptr();
                        let indices_u8 = slice::from_raw_parts(buffer_ptr as *const u8, self.shape_buffer_indices_count * 4);

                        self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, models_u8);
                        self.gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, 0, indices_u8);

                        if let Some(texture_id) = buffer_metadata.texture_id {
                            self.textures.get(texture_id)?.activate();
                        } else {
                            self.textures.get(self.default_texture_id)?.activate();
                        }

                        self.gl.draw_elements(glow::TRIANGLES, self.shape_buffer_indices_count as i32, glow::UNSIGNED_INT, 0);

                        self.shape_buffer_vertices_count = 0;
                        self.shape_buffer_indices_count = 0;
                        self.shape_buffer_indices_max = 0;
                    }
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

impl BufferMetadata {
    pub fn new(content_type: BufferContentType, texture_id: Option<usize>) -> Self {
        Self { content_type, texture_id }
    }
}
