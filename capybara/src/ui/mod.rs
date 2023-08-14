use crate::renderer::texture::{Texture, TextureKind};
use anyhow::bail;
use anyhow::Result;
use egui::{Image, Pos2, Rect, Vec2};

pub mod context;

pub trait ImageAtlas {
    fn atlas(&mut self, texture: &Texture, entity_name: &str) -> Result<Self>
    where
        Self: Sized;
}

impl ImageAtlas for Image {
    fn atlas(&mut self, texture: &Texture, entity_name: &str) -> Result<Self> {
        if let TextureKind::Atlas(entities) = &texture.kind {
            if let Some(entity) = entities.get(entity_name) {
                let position = entity.position / texture.size;
                let size = entity.size / texture.size;

                return Ok(self.uv(Rect::from_min_size(Pos2::new(position.x, position.y), Vec2::new(size.x, size.y))));
            }
        }

        bail!("Can't use this texture as atlas")
    }
}
