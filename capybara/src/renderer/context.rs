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
use crate::error_continue;
use crate::error_return;
use crate::utils::color::Vec4Color;
use crate::utils::storage::Storage;
use anyhow::Error;
use anyhow::Result;
use glam::Vec2;
use glam::Vec4;
use glow::Buffer;
use glow::Context;
use glow::HasContext;
use glow::VertexArray;
use instant::Instant;
use log::error;
use log::info;
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

    active_camera_data: Camera,
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
            let sprite_buffer_vao = gl.create_vertex_array().map_err(Error::msg)?;
            let sprite_buffer_base_vbo = gl.create_buffer().map_err(Error::msg)?;
            let sprite_buffer_data_vbo = gl.create_buffer().map_err(Error::msg)?;
            let sprite_buffer_ebo = gl.create_buffer().map_err(Error::msg)?;

            let shape_buffer_vao = gl.create_vertex_array().map_err(Error::msg)?;
            let shape_buffer_vbo = gl.create_buffer().map_err(Error::msg)?;
            let shape_buffer_ebo = gl.create_buffer().map_err(Error::msg)?;

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

                active_camera_data: Default::default(),
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

            context.gl.enable(glow::BLEND);
            context.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            context.set_clear_color(Vec4::new(0.0, 0.0, 0.0, 1.0));

            let camera = Camera::new(Default::default(), Default::default(), CameraOrigin::LeftBottom);
            context.default_camera_id = context.cameras.store(camera);
            context.set_camera(context.default_camera_id);

            let sprite_shader = Shader::new(&context, "sprite_default", SPRITE_VERTEX_SHADER, SPRITE_FRAGMENT_SHADER)?;
            context.default_sprite_shader_id = context.shaders.store(sprite_shader);

            let shape_shader = Shader::new(&context, "shape_default", SHAPE_VERTEX_SHADER, SHAPE_FRAGMENT_SHADER)?;
            context.default_shape_shader_id = context.shaders.store(shape_shader);

            let default_texture = Texture::new(&context, &RawTexture::new("blank", "", Vec2::new(1.0, 1.0), &[255, 255, 255, 255]))?;
            context.default_texture_id = context.textures.store(default_texture);

            // Sprite buffers
            context.gl.bind_vertex_array(Some(context.sprite_buffer_vao));
            context.gl.bind_buffer(glow::ARRAY_BUFFER, Some(context.sprite_buffer_base_vbo));

            context.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(context.sprite_buffer_ebo));
            context.gl.enable_vertex_attrib_array(0);
            context.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 2 * 4, 0);

            let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
            let indices = [0, 1, 2, 0, 2, 3];

            let models_u8 = core::slice::from_raw_parts(vertices.as_ptr() as *const u8, 8 * 4);
            let indices_u8 = core::slice::from_raw_parts(indices.as_ptr() as *const u8, 6 * 4);

            context.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, models_u8, glow::STATIC_DRAW);
            context.gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STATIC_DRAW);

            context.gl.bind_buffer(glow::ARRAY_BUFFER, Some(context.sprite_buffer_data_vbo));

            context.gl.enable_vertex_attrib_array(1);
            context.gl.enable_vertex_attrib_array(2);
            context.gl.enable_vertex_attrib_array(3);
            context.gl.enable_vertex_attrib_array(4);
            context.gl.enable_vertex_attrib_array(5);
            context.gl.enable_vertex_attrib_array(6);

            context.gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 12 * 4, 0);
            context.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 12 * 4, 2 * 4);
            context.gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, 12 * 4, 4 * 4);
            context.gl.vertex_attrib_pointer_f32(4, 2, glow::FLOAT, false, 12 * 4, 5 * 4);
            context.gl.vertex_attrib_pointer_i32(5, 4, glow::UNSIGNED_BYTE, 12 * 4, 7 * 4);
            context.gl.vertex_attrib_pointer_f32(6, 4, glow::FLOAT, false, 12 * 4, 8 * 4);

            context.gl.vertex_attrib_divisor(1, 1);
            context.gl.vertex_attrib_divisor(2, 1);
            context.gl.vertex_attrib_divisor(3, 1);
            context.gl.vertex_attrib_divisor(4, 1);
            context.gl.vertex_attrib_divisor(5, 1);
            context.gl.vertex_attrib_divisor(6, 1);

            // UI buffers
            context.gl.bind_vertex_array(Some(context.shape_buffer_vao));
            context.gl.bind_buffer(glow::ARRAY_BUFFER, Some(context.shape_buffer_vbo));
            context.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(context.shape_buffer_ebo));

            context.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 5 * 4, 0);
            context.gl.vertex_attrib_pointer_i32(1, 4, glow::UNSIGNED_BYTE, 5 * 4, 2 * 4);
            context.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 5 * 4, 3 * 4);

            context.gl.enable_vertex_attrib_array(0);
            context.gl.enable_vertex_attrib_array(1);
            context.gl.enable_vertex_attrib_array(2);

            Ok(context)
        }
    }

    pub fn instantiate_assets(&mut self, assets: &AssetsLoader, prefix: Option<&str>) {
        info!("Instancing renderer assets, prefix {}", prefix.unwrap_or("none"));

        for raw in &assets.raw_textures {
            if let Some(prefix) = &prefix {
                if !raw.path.starts_with(prefix) {
                    continue;
                }
            }

            let texture = match Texture::new(self, raw) {
                Ok(sound) => sound,
                Err(err) => error_continue!("Failed to load texture {} ({})", raw.name, err),
            };

            if let Err(err) = self.textures.store_with_name(&raw.name, texture) {
                error!("Failed to instantiate texture {} ({})", raw.name, err);
            }
        }

        for raw in &assets.raw_atlases {
            let path = Path::new(&raw.name);
            let name = match path.file_stem() {
                Some(name) => name,
                None => error_continue!("Failed to get filename stem for atlas {}", raw.name),
            };
            let name_str = match name.to_str() {
                Some(name) => name,
                None => error_continue!("Failed to get filename string for atlas {}", raw.name),
            };

            if self.textures.contains_by_name(name_str) {
                let mut entities = FxHashMap::default();
                let texture = match self.textures.get_by_name_mut(name_str) {
                    Ok(texture) => texture,
                    Err(err) => error_continue!("{}, atlas {} orphaned", err, name_str),
                };

                for entity in &raw.entities {
                    entities.insert(entity.name.clone(), AtlasEntity::new(entity.position, entity.size));
                }

                texture.kind = TextureKind::Atlas(entities);
            }
        }
    }

    pub fn begin_frame(&mut self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            if self.active_camera_id != self.default_camera_id {
                self.set_camera(self.default_camera_id);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.flush_buffer();

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
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite) {
        let camera = match self.cameras.get(self.active_camera_id) {
            Ok(camera) => camera,
            Err(err) => error_return!("Failed to draw sprite ({})", err),
        };

        let sprite_size = if let Some(texture_id) = sprite.texture_id {
            let texture = match self.textures.get(texture_id) {
                Ok(texture) => texture,
                Err(err) => error_return!("Failed to draw sprite ({})", err),
            };

            match &sprite.texture_type {
                TextureType::Simple => sprite.size.unwrap_or(texture.size),
                TextureType::SimpleOffset { offset: _ } => sprite.size.unwrap_or(texture.size),
                TextureType::Tilemap { size } => sprite.size.unwrap_or(*size),
                TextureType::TilemapAnimation { size, frames: _ } => sprite.size.unwrap_or(*size),
                TextureType::AtlasEntity { name } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let entity = match atlas_entities.get(name) {
                            Some(entity) => entity,
                            None => error_return!("Entity {} not found", name),
                        };
                        sprite.size.unwrap_or(entity.size)
                    } else {
                        error_return!("Texture {} is not an atlas", texture_id);
                    }
                }
                TextureType::AtlasAnimation { entities } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let name = &entities[sprite.animation_frame];
                        let entity = match atlas_entities.get(name) {
                            Some(entity) => entity,
                            None => error_return!("Entity {} not found", name),
                        };
                        sprite.size.unwrap_or(entity.size)
                    } else {
                        error_return!("Texture {} is not an atlas", texture_id);
                    }
                }
            }
        } else {
            sprite.size.unwrap_or(Vec2::new(1.0, 1.0))
        };

        let camera_min = camera.position;
        let camera_max = camera.position + camera.size;

        let radius = sprite.anchor.length() + 2.0;
        let sprite_min = sprite.position - radius * sprite_size.max_element() * sprite.scale.max_element();
        let sprite_max = sprite.position + radius * sprite_size.max_element() * sprite.scale.max_element();

        if sprite_min.x > camera_max.x || sprite_min.y > camera_max.y || sprite_max.x < camera_min.x || sprite_max.y < camera_min.y {
            return;
        }

        if let Some(buffer_metadata) = &self.buffer_metadata {
            if buffer_metadata.content_type != BufferContentType::Sprite || buffer_metadata.texture_id != sprite.texture_id {
                self.flush_buffer();
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
            let texture = match self.textures.get(texture_id) {
                Ok(texture) => texture,
                Err(err) => error_return!("Failed to draw sprite ({})", err),
            };

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
                        let entity = match atlas_entities.get(name) {
                            Some(entity) => entity,
                            None => error_return!("Entity {} not found", name),
                        };
                        (entity.position / texture.size, entity.size / texture.size)
                    } else {
                        error_return!("Texture {} is not an atlas", texture_id);
                    }
                }
                TextureType::AtlasAnimation { entities } => {
                    if let TextureKind::Atlas(atlas_entities) = &texture.kind {
                        let name = &entities[sprite.animation_frame];
                        let entity = match atlas_entities.get(name) {
                            Some(entity) => entity,
                            None => error_return!("Entity {} not found", name),
                        };

                        (entity.position / texture.size, entity.size / texture.size)
                    } else {
                        error_return!("Texture {} is not an atlas", texture_id);
                    }
                }
            }
        } else {
            (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))
        };

        let color = sprite.color.to_rgb_packed();

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
    }

    pub fn draw_shape(&mut self, shape: &Shape) {
        if let Some(buffer_metadata) = &self.buffer_metadata {
            if buffer_metadata.content_type != BufferContentType::Shape || buffer_metadata.texture_id != shape.texture_id {
                self.flush_buffer();
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

            for i in (self.shape_buffer_vertices_count..(self.shape_buffer_vertices_count + shape.vertices.len())).step_by(5) {
                let x = self.shape_buffer_vertices_queue[i + 0];
                let y = self.shape_buffer_vertices_queue[i + 1];
                let position = Vec4::new(f32::from_bits(x), f32::from_bits(y), 0.0, 1.0);
                let position_transformed = model * position;

                self.shape_buffer_vertices_queue[i + 0] = position_transformed.x.to_bits();
                self.shape_buffer_vertices_queue[i + 1] = position_transformed.y.to_bits();
            }
        }

        let base_indice = self.shape_buffer_indices_max;
        for i in 0..shape.indices.len() {
            self.shape_buffer_indices_queue[self.shape_buffer_indices_count + i] = base_indice + shape.indices[i];
            self.shape_buffer_indices_max = cmp::max(self.shape_buffer_indices_max, base_indice + shape.indices[i] + 1);
        }

        self.shape_buffer_vertices_count += shape.vertices.len();
        self.shape_buffer_indices_count += shape.indices.len();
    }

    pub fn flush_buffer(&mut self) {
        unsafe {
            if let Some(buffer_metadata) = &self.buffer_metadata {
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

                let camera = match self.cameras.get_mut(self.active_camera_id) {
                    Ok(camera) => camera,
                    Err(err) => error_return!("Failed to flush buffer ({})", err),
                };
                let camera_changed = *camera != self.active_camera_data;

                match buffer_metadata.content_type {
                    BufferContentType::Sprite => {
                        if self.active_shader_id != self.default_sprite_shader_id || camera_changed {
                            match self.shaders.get(self.default_sprite_shader_id) {
                                Ok(shader) => {
                                    shader.activate();
                                    shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr());
                                    shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr());
                                }
                                Err(err) => error!("{}", err),
                            }

                            if camera_changed {
                                self.active_camera_data = camera.clone();
                            }

                            self.active_shader_id = self.default_sprite_shader_id;
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
                            match self.textures.get(texture_id) {
                                Ok(texture) => texture.activate(),
                                Err(err) => error!("{}", err),
                            };
                        } else {
                            match self.textures.get(self.default_texture_id) {
                                Ok(texture) => texture.activate(),
                                Err(err) => error!("{}", err),
                            };
                        }

                        self.gl.draw_elements_instanced(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0, self.sprite_buffer_count as i32);

                        self.sprite_buffer_count = 0;
                        self.sprite_buffer_vertices_count = 0;
                    }
                    BufferContentType::Shape => {
                        if self.active_shader_id != self.default_shape_shader_id || camera_changed {
                            match self.shaders.get(self.default_shape_shader_id) {
                                Ok(shader) => {
                                    shader.activate();
                                    shader.set_uniform("proj", camera.get_projection_matrix().as_ref().as_ptr());
                                    shader.set_uniform("view", camera.get_view_matrix().as_ref().as_ptr());
                                }
                                Err(err) => error!("{}", err),
                            }

                            if camera_changed {
                                self.active_camera_data = camera.clone();
                            }

                            self.active_shader_id = self.default_shape_shader_id;
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
                            match self.textures.get(texture_id) {
                                Ok(texture) => texture.activate(),
                                Err(err) => error!("{}", err),
                            };
                        } else {
                            match self.textures.get(self.default_texture_id) {
                                Ok(texture) => texture.activate(),
                                Err(err) => error!("{}", err),
                            };
                        }

                        self.gl.draw_elements(glow::TRIANGLES, self.shape_buffer_indices_count as i32, glow::UNSIGNED_INT, 0);

                        self.shape_buffer_vertices_count = 0;
                        self.shape_buffer_indices_count = 0;
                        self.shape_buffer_indices_max = 0;
                    }
                }

                self.buffer_metadata = None;
            }
        }
    }

    pub fn set_camera(&mut self, camera_id: usize) {
        let camera = match self.cameras.get_mut(camera_id) {
            Ok(camera) => camera,
            Err(err) => error_return!("Failed to set camera ({})", err),
        };

        self.active_camera_id = camera_id;
        camera.size = self.viewport_size;
    }

    pub fn set_viewport(&mut self, size: Vec2) {
        unsafe {
            self.gl.viewport(0, 0, size.x as i32, size.y as i32);
            self.viewport_size = size;

            let camera = match self.cameras.get_mut(self.active_camera_id) {
                Ok(camera) => camera,
                Err(err) => error_return!("Failed to set viewport ({})", err),
            };
            camera.size = self.viewport_size;
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
