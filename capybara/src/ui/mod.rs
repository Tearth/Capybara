use crate::renderer::texture::Texture;
use crate::renderer::texture::TextureKind;
use anyhow::bail;
use anyhow::Result;
use egui::Image;
use egui::Pos2;
use egui::Rect;
use egui::TextureHandle;
use egui::Vec2;

pub mod context;
pub mod widgets;

pub trait ImageAtlas {
    fn from_atlas(handle: &TextureHandle, texture: &Texture, entity_name: &str) -> Result<Self>
    where
        Self: Sized;
}

impl ImageAtlas for Image<'_> {
    fn from_atlas(handle: &TextureHandle, texture: &Texture, entity_name: &str) -> Result<Self> {
        if let TextureKind::Atlas(entities) = &texture.kind {
            if let Some(entity) = entities.get(entity_name) {
                let position = entity.position / texture.size;
                let size = entity.size / texture.size;

                let image = Image::from_texture((handle.id(), Vec2::new(entity.size.x, entity.size.y)));
                let image = image.uv(Rect::from_min_size(Pos2::new(position.x, position.y), Vec2::new(size.x, size.y)));

                return Ok(image);
            }
        }

        bail!("Can't use this texture as atlas")
    }
}
