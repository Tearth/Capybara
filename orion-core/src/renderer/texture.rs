use crate::{assets::RawTexture, utils::storage::StorageItem};
use glow::HasContext;
use std::rc::Rc;

pub struct Texture {
    pub id: usize,
    pub name: Option<String>,
    pub inner: glow::Texture,
    gl: Rc<glow::Context>,
}

impl Texture {
    pub fn new(gl: Rc<glow::Context>, raw: &RawTexture) -> Self {
        unsafe {
            let inner = gl.create_texture().unwrap();

            gl.bind_texture(glow::TEXTURE_2D, Some(inner));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::MIRRORED_REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::MIRRORED_REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST_MIPMAP_NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA8 as i32, raw.size.x as i32, raw.size.y as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, Some(&raw.data));
            gl.generate_mipmap(glow::TEXTURE_2D);

            Self { id: 0, name: None, inner, gl }
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
