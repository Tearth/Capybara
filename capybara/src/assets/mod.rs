use glam::Vec2;

pub mod loader;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AssetsLoadingStatus {
    Idle,
    Loading,
    Parsing,
    Finished,
    Error,
}

pub struct RawTexture {
    pub name: String,
    pub path: String,
    pub size: Vec2,
    pub data: Vec<u8>,
}

pub struct RawFont {
    pub name: String,
    pub path: String,
    pub data: Vec<u8>,
}

pub struct RawAtlas {
    pub name: String,
    pub path: String,
    pub image: String,
    pub entities: Vec<RawAtlasEntity>,
}

pub struct RawAtlasEntity {
    pub name: String,
    pub path: String,
    pub position: Vec2,
    pub size: Vec2,
}

pub struct RawSound {
    pub name: String,
    pub path: String,
    pub data: Vec<u8>,
}

impl RawTexture {
    pub fn new(name: &str, path: &str, size: Vec2, data: &[u8]) -> Self {
        Self { name: name.to_string(), path: path.to_string(), size, data: data.to_vec() }
    }
}

impl RawFont {
    pub fn new(name: &str, path: &str, data: &[u8]) -> Self {
        Self { name: name.to_string(), path: path.to_string(), data: data.to_vec() }
    }
}

impl RawAtlas {
    pub fn new(name: &str, path: &str, image: &str, textures: Vec<RawAtlasEntity>) -> Self {
        Self { name: name.to_string(), path: path.to_string(), image: image.to_string(), entities: textures }
    }
}

impl RawAtlasEntity {
    pub fn new(name: &str, path: &str, position: Vec2, size: Vec2) -> Self {
        Self { name: name.to_string(), path: path.to_string(), position, size }
    }
}

impl RawSound {
    pub fn new(name: &str, path: &str, data: &[u8]) -> Self {
        Self { name: name.to_string(), path: path.to_string(), data: data.to_vec() }
    }
}
