use glam::Vec2;

pub mod bundler;
pub mod loader;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AssetsLoadingStatus {
    Idle,
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

impl RawTexture {
    pub fn new(name: String, size: Vec2, data: &[u8]) -> Self {
        Self { name, size, data: data.to_vec() }
    }
}

impl RawFont {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }
}
