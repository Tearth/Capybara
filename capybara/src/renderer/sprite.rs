use super::*;
use arrayvec::ArrayVec;
use glam::Vec2;
use glam::Vec4;
use instant::Instant;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Option<Vec2>,
    pub anchor: Vec2,
    pub color: Vec4,
    pub texture_id: TextureId,
    pub texture_type: TextureType,
    pub rounded_coordinates: bool,

    pub animation_frame: i32,
    pub animation_speed: f32,
    pub animation_loop: bool,
    pub animation_backward: bool,
    pub animation_timestamp: Instant,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SpriteVertex {
    pub position: Vec2,
    pub anchor: Vec2,
    pub rotation: f32,
    pub size: Vec2,
    pub color: u32,
    pub uv_position: Vec2,
    pub uv_size: Vec2,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum TextureId {
    #[default]
    Default,
    Some(usize),
    None,
}

#[rustfmt::skip]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TextureType {
    #[default]
    Simple,
    SimpleOffset { offset: Vec2 },
    SimpleCoordinates { position: Vec2, size: Vec2 },
    Tilemap { size: Vec2 },
    TilemapAnimation { size: Vec2, frames: Vec<usize> },
    AtlasEntity { name: String },
    AtlasAnimation { entities: Vec<String> },
}

impl Sprite {
    pub fn new() -> Self {
        Self {
            position: Default::default(),
            rotation: 0.0,
            scale: Vec2::ONE,
            size: Default::default(),
            anchor: Vec2::new(0.5, 0.5),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            texture_id: TextureId::Default,
            texture_type: TextureType::Simple,
            rounded_coordinates: false,

            animation_frame: 0,
            animation_speed: 1.0,
            animation_loop: true,
            animation_backward: false,
            animation_timestamp: Instant::now(),
        }
    }

    pub fn get_edges(&self, texture_size: Vec2) -> ArrayVec<Edge, 4> {
        let size = self.size.unwrap_or(texture_size) * self.scale;
        let a = Vec2::new(0.0, 0.0) - size * self.anchor;
        let b = Vec2::new(size.x, 0.0) - size * self.anchor;
        let c = Vec2::new(size.x, size.y) - size * self.anchor;
        let d = Vec2::new(0.0, size.y) - size * self.anchor;

        let sin = self.rotation.sin();
        let cos = self.rotation.cos();

        let a = Vec2::new(a.x * cos - a.y * sin, a.y * cos + a.x * sin) + self.position;
        let b = Vec2::new(b.x * cos - b.y * sin, b.y * cos + b.x * sin) + self.position;
        let c = Vec2::new(c.x * cos - c.y * sin, c.y * cos + c.x * sin) + self.position;
        let d = Vec2::new(d.x * cos - d.y * sin, d.y * cos + d.x * sin) + self.position;

        ArrayVec::from([Edge::new(a, b), Edge::new(d, c), Edge::new(a, d), Edge::new(b, c)])
    }

    pub fn is_animation(&self) -> bool {
        matches!(self.texture_type, TextureType::TilemapAnimation { size: _, frames: _ } | TextureType::AtlasAnimation { entities: _ })
    }

    pub fn animate(&mut self, now: Instant) {
        let frames_count = match &self.texture_type {
            TextureType::TilemapAnimation { size: _, frames } => frames.len(),
            TextureType::AtlasAnimation { entities } => entities.len(),
            _ => return,
        } as i32;

        if self.animation_frame == frames_count - 1 && !self.animation_loop {
            return;
        }

        if (now - self.animation_timestamp).as_secs_f32() >= self.animation_speed / 1000.0 {
            if !self.animation_backward {
                self.animation_frame = (self.animation_frame + 1) % frames_count;
            } else {
                self.animation_frame -= 1;
                if self.animation_frame < 0 {
                    self.animation_frame += frames_count;
                }
            }

            self.animation_timestamp = now;
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}
