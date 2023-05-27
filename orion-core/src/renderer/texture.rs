use super::context::RendererContext;
use crate::assets::RawTexture;
use crate::utils::storage::StorageItem;
use glam::Vec2;
use glow::Context;
use glow::HasContext;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Texture {
    pub id: usize,
    pub name: Option<String>,
    pub size: Vec2,
    pub inner: glow::Texture,
    pub kind: TextureKind,
    gl: Rc<Context>,
}

pub struct AtlasEntity {
    pub position: Vec2,
    pub size: Vec2,
}

pub enum TextureKind {
    Simple,
    Atlas(FxHashMap<String, AtlasEntity>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

impl Texture {
    pub fn new(renderer: &RendererContext, raw: &RawTexture) -> Self {
        unsafe {
            let gl = renderer.gl.clone();
            let inner = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(inner));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST_MIPMAP_NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8_ALPHA8 as i32,
                raw.size.x as i32,
                raw.size.y as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&raw.data),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);

            Self { id: 0, name: None, size: raw.size, inner, kind: TextureKind::Simple, gl }
        }
    }

    pub fn update(&self, position: Vec2, size: Vec2, data: &[u8]) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.inner));
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                position.x as i32,
                position.y as i32,
                size.x as i32,
                size.y as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(data),
            );
            self.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn set_filters(&self, minification: TextureFilter, magnification: TextureFilter) {
        let minification_value = match minification {
            TextureFilter::Linear => glow::LINEAR_MIPMAP_LINEAR,
            TextureFilter::Nearest => glow::NEAREST_MIPMAP_NEAREST,
        } as i32;

        let magnification_value = match magnification {
            TextureFilter::Linear => glow::LINEAR,
            TextureFilter::Nearest => glow::NEAREST,
        } as i32;

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.inner));
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, minification_value);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, magnification_value);
            self.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }

    pub fn activate(&self) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.inner));
        }
    }
}

impl StorageItem for Texture {
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.inner);
        }
    }
}

impl AtlasEntity {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }
}
