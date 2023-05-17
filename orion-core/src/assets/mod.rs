use glam::Vec2;

pub mod bundler;
pub mod loader;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AssetsLoadingStatus {
    Idle,
    Initializing,
    Loading,
    Parsing,
    Finished,
}

pub struct RawTexture {
    pub name: String,
    pub size: Vec2,
    pub data: Vec<u8>,
}

pub struct RawFont {
    pub name: String,
    pub data: Vec<u8>,
}

pub struct RawAtlas {
    pub name: String,
    pub image: String,
    pub textures: Vec<RawAtlasTexture>,
}

pub struct RawAtlasTexture {
    pub name: String,
    pub position: Vec2,
    pub size: Vec2,
}

impl RawTexture {
    pub fn new(name: &str, size: Vec2, data: &[u8]) -> Self {
        Self { name: name.to_string(), size, data: data.to_vec() }
    }
}

impl RawFont {
    pub fn new(name: &str, data: &[u8]) -> Self {
        Self { name: name.to_string(), data: data.to_vec() }
    }
}

impl RawAtlas {
    pub fn new(name: &str, image: &str, textures: Vec<RawAtlasTexture>) -> Self {
        Self { name: name.to_string(), image: image.to_string(), textures }
    }
}

impl RawAtlasTexture {
    pub fn new(name: &str, position: Vec2, size: Vec2) -> Self {
        Self { name: name.to_string(), position, size }
    }
}
