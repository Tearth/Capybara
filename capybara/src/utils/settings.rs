use crate::error_return;
use crate::filesystem::FileSystem;
use anyhow::anyhow;
use anyhow::Result;
use rustc_hash::FxHashMap;
use std::str::FromStr;

pub struct SettingsStorage {
    path: String,
    filesystem: FileSystem,
    cache: FxHashMap<String, String>,
}

impl SettingsStorage {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), filesystem: Default::default(), cache: Default::default() }
    }

    pub fn get<T>(&mut self, key: &str) -> Result<T>
    where
        T: FromStr + ToString,
    {
        if !self.cache.is_empty() {
            if let Some(value) = self.cache.get(key).cloned() {
                return value.parse().map_err(|_| anyhow!("Failed to parse {}", value));
            }
        }

        let content = self.filesystem.read_local(&self.path)?;
        self.cache = self.deserialize(&content)?;

        let value = self.cache.get(key).cloned().ok_or_else(|| anyhow!("Key not found"))?;
        value.parse().map_err(|_| anyhow!("Failed to parse {}", value))
    }

    pub fn set<T>(&mut self, key: &str, value: T, overwrite: bool)
    where
        T: FromStr + ToString,
    {
        if self.cache.is_empty() {
            if let Ok(content) = self.filesystem.read_local(&self.path) {
                self.cache = match self.deserialize(&content) {
                    Ok(cache) => cache,
                    Err(err) => error_return!("Failed to deserialize settings ({})", err),
                };
            }
        }

        if self.cache.get(key).is_none() || overwrite {
            self.cache.insert(key.to_string(), value.to_string());
            self.filesystem.write_local(&self.path, &self.serialize(&self.cache));
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
