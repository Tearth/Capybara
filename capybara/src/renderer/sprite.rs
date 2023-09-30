use glam::Vec2;
use glam::Vec4;
use instant::Instant;

pub struct Sprite {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Option<Vec2>,
    pub anchor: Vec2,
    pub color: Vec4,
    pub texture_id: TextureId,
    pub texture_type: TextureType,

    pub animation_frame: usize,
    pub animation_speed: f32,
    pub animation_loop: bool,
    pub animation_timestamp: Instant,
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

            animation_frame: 0,
            animation_speed: 1.0,
            animation_loop: true,
            animation_timestamp: Instant::now(),
        }
    }

    pub fn is_animation(&self) -> bool {
        matches!(self.texture_type, TextureType::TilemapAnimation { size: _, frames: _ } | TextureType::AtlasAnimation { entities: _ })
    }

    pub fn animate(&mut self, now: Instant) {
        let frames_count = match &self.texture_type {
            TextureType::TilemapAnimation { size: _, frames } => frames.len(),
            TextureType::AtlasAnimation { entities } => entities.len(),
            _ => return,
        };

        if self.animation_frame == frames_count - 1 && !self.animation_loop {
            return;
        }

        if (now - self.animation_timestamp).as_millis() >= (1000.0 / self.animation_speed) as u128 {
            self.animation_frame = (self.animation_frame + 1) % frames_count;
            self.animation_timestamp = now;
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}
