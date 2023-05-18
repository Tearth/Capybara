use glam::{Mat4, Vec2, Vec3, Vec4};

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
}

pub enum Shape {
    Standard,
    Custom(ShapeData),
}

pub struct ShapeData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

pub enum Tile {
    Simple,
    AtlasEntity(String),
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
        }
    }

    pub fn get_model(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(self.size.x * self.scale.x, self.size.y * self.scale.y, 0.0));
        let anchor = Mat4::from_translation(-Vec3::new(self.anchor.x, self.anchor.y, 0.0));

        translation * rotation * scale * anchor
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
