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

impl RawTexture {
    pub fn new(name: String, size: Vec2, data: Vec<u8>) -> Self {
        Self { name, size, data }
    }
}
