use crate::filesystem::FileSystem;
use anyhow::anyhow;
use anyhow::Result;
use rustc_hash::FxHashMap;
use std::str::FromStr;

pub struct SettingsStorage {
    path: String,
    filesystem: FileSystem,
    cache: Option<FxHashMap<String, String>>,
}

impl SettingsStorage {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), filesystem: Default::default(), cache: None }
    }

    pub fn get<T>(&mut self, key: &str) -> Result<Option<T>>
    where
        T: FromStr,
    {
        if let Some(cache) = &self.cache {
            if let Some(value) = cache.get(key).cloned() {
                return Ok(value.parse().ok());
            }
        }

        let content = self.filesystem.read_local(&self.path).unwrap();
        let settings = self.deserialize(&content)?;
        self.cache = Some(settings);

        Ok(self.cache.as_ref().unwrap().get(key).cloned().ok_or_else(|| anyhow!("Key not found"))?.parse().ok())
    }

    pub fn set<T>(&mut self, key: &str, value: T, overwrite: bool) -> Result<Option<T>>
    where
        T: FromStr + ToString,
    {
        if self.cache.is_none() {
            if let Ok(content) = self.filesystem.read_local(&self.path) {
                let settings = self.deserialize(&content)?;
                self.cache = Some(settings);
            } else {
                self.cache = Some(FxHashMap::default());
            }
        }

        if self.cache.as_ref().unwrap().get(key).is_none() || overwrite {
            self.cache.as_mut().unwrap().insert(key.to_string(), value.to_string());
            self.filesystem.write_local(&self.path, &self.serialize(self.cache.as_ref().unwrap()))?;

            Ok(Some(value))
        } else {
            Ok(self.cache.as_ref().unwrap().get(key).cloned().ok_or_else(|| anyhow!("Key not found"))?.parse().ok())
        }
    }

    fn serialize(&self, settings: &FxHashMap<String, String>) -> String {
        let mut output = String::new();

        for item in settings {
            output.push_str(&format!("{}={}\n", item.0, item.1));
        }

        output.trim().to_string()
    }

    fn deserialize(&self, settings: &str) -> Result<FxHashMap<String, String>> {
        let mut output = FxHashMap::default();

        for line in settings.lines().map(|p| p.trim()) {
            let tokens = line.split('=').collect::<Vec<&str>>();
            let name = tokens[0].trim();
            let value = tokens[1].trim();

            output.insert(name.to_string(), value.to_string());
        }

        Ok(output)
    }
}
