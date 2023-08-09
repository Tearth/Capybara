use anyhow::Result;
use rustc_hash::FxHashMap;
use std::fs;

pub struct SettingsStorageNative {
    pub(crate) path: String,
    pub(crate) cache: Option<FxHashMap<String, String>>,
}

impl SettingsStorageNative {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), cache: None }
    }

    pub(crate) fn read_content(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.path)?)
    }

    pub(crate) fn write_content(&self, content: &str) -> Result<()> {
        Ok(fs::write(&self.path, content)?)
    }
}
