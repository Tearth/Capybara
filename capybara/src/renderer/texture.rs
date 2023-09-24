use super::context::RendererContext;
use crate::assets::RawTexture;
use anyhow::Error;
use anyhow::Result;
use glam::Vec2;
use glow::Context;
use glow::HasContext;
use log::info;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Texture {
    pub name: String,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TextureWrapMode {
    Repeat,
    Clamp,
}

impl Texture {
    pub fn new(renderer: &RendererContext, raw: &RawTexture) -> Result<Self> {
        unsafe {
            info!("Creating texture {} ({}x{}, {} bytes)", raw.name, raw.size.x, raw.size.y, raw.data.len());

            let gl = renderer.gl.clone();
            let inner = gl.create_texture().map_err(Error::msg)?;
            let data = if raw.data.len() != 0 { Some(&raw.data) } else { None };

            gl.bind_texture(glow::TEXTURE_2D, Some(inner));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8_ALPHA8 as i32,
                raw.size.x as i32,
                raw.size.y as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                data.map(|p| p.as_slice()),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);

            Ok(Self { name: raw.name.to_string(), size: raw.size, inner, kind: TextureKind::Simple, gl })
        }
    }

    pub fn update(&self, position: Vec2, size: Vec2, data: &[u8]) {
        unsafe {
            info!("Updating texture {} ({}x{}, {} bytes)", self.name, size.x, size.y, data.len());

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

    pub fn resize(&mut self, size: Vec2) {
        unsafe {
            info!("Resizing texture {} ({}x{})", self.name, size.x, size.y);

            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.inner));
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8_ALPHA8 as i32,
                size.x as i32,
                size.y as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
            self.gl.generate_mipmap(glow::TEXTURE_2D);
            self.size = size;
        }
    }

    pub fn set_filters(&self, minification: TextureFilter, magnification: TextureFilter) {
        info!("Updating texture {} (minification {:?}, magnification {:?})", self.name, minification, magnification);

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

    pub fn set_wrap_mode(&self, mode: TextureWrapMode) {
        info!("Updating texture {} (wrap mode {:?})", self.name, mode);

        let value = match mode {
            TextureWrapMode::Repeat => glow::REPEAT,
            TextureWrapMode::Clamp => glow::CLAMP_TO_EDGE,
        } as i32;

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.inner));
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, value);
            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, value);
        }
    }

    pub fn activate(&self) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.inner));
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            info!("Releasing texture {}", self.name);
            self.gl.delete_texture(self.inner);
        }
    }
}

impl AtlasEntity {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }
}
