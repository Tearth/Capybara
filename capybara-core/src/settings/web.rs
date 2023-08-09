use anyhow::bail;
use anyhow::Result;
use rustc_hash::FxHashMap;
use web_sys::Storage;

pub struct SettingsStorageWeb {
    pub(crate) path: String,
    pub(crate) cache: Option<FxHashMap<String, String>>,
    storage: Storage,
}

impl SettingsStorageWeb {
    pub fn new(path: &str) -> Self {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        Self { path: path.to_string(), cache: None, storage }
    }

    pub(crate) fn read_content(&self) -> Result<String> {
        if let Ok(settings) = self.storage.get(&self.path) {
            if let Some(settings) = settings {
                return Ok(settings);
            } else {
                return Ok("".to_string());
            }
        }

        bail!("Local storate is not available")
    }

    pub(crate) fn write_content(&self, content: &str) -> Result<()> {
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();

        storage.set(&self.path, content).unwrap();

        Ok(())
    }
}
