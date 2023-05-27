use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::Vec4;
use instant::Instant;

pub struct Sprite {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub size: Vec2,
    pub anchor: Vec2,
    pub color: Vec4,
    pub shape: Shape,
    pub texture_id: usize,
    pub tile: Tile,

    pub animation_frame: usize,
    pub animation_speed: f32,
    pub animation_loop: bool,
    pub animation_timestamp: Instant,
}

#[derive(Clone, Debug)]
pub enum Shape {
    Standard,
    Custom(ShapeData),
}

#[derive(Clone, Debug)]
pub struct ShapeData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tile {
    Simple,
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
            shape: Shape::Standard,
            texture_id: 0,
            tile: Tile::Simple,

            animation_frame: 0,
            animation_speed: 1.0,
            animation_loop: true,
            animation_timestamp: Instant::now(),
        }
    }

    pub fn get_model(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(self.size.x * self.scale.x, self.size.y * self.scale.y, 0.0));
        let anchor = Mat4::from_translation(-Vec3::new(self.anchor.x, self.anchor.y, 0.0));

        translation * rotation * scale * anchor
    }

    pub fn animate(&mut self, now: Instant) {
        let frames_count = match &self.tile {
            Tile::TilemapAnimation { size: _, frames } => frames.len(),
            Tile::AtlasAnimation { entities } => entities.len(),
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

impl ShapeData {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}
