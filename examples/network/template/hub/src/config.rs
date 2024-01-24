use capybara::anyhow::Result;
use capybara::error_return;
use capybara::utils::json::*;
use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str;
use tinyjson::JsonValue;

#[derive(Debug, Default, Clone)]
pub struct ConfigLoader {
    pub path: String,
    pub data: ConfigData,
}

#[derive(Debug, Default, Clone)]
pub struct ConfigData {
    pub endpoint: String,
    pub tick: u32,
    pub servers: Vec<ConfigServerData>,
}

#[derive(Debug, Default, Clone)]
pub struct ConfigServerData {
    pub name: String,
    pub flag: String,
    pub address: String,
    pub enabled: bool,
}

impl ConfigLoader {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string(), ..Default::default() }
    }

    pub fn reload(&mut self) {
        self.data = Default::default();

        let mut file = match File::open(&self.path) {
            Ok(file) => file,
            Err(err) => error_return!("Failed to open file ({})", err),
        };

        let mut buffer = Vec::new();
        if let Err(err) = file.read_to_end(&mut buffer) {
            error_return!("Failed to read file ({})", err);
        }

        let content = match str::from_utf8(&buffer) {
            Ok(content) => content,
            Err(err) => error_return!("Failed to parse content ({})", err),
        };

        let json = match content.parse::<JsonValue>() {
            Ok(json) => json,
            Err(err) => error_return!("Failed to parse JSON ({})", err),
        };

        let data = match json.get::<HashMap<_, _>>() {
            Some(data) => data,
            None => error_return!("Failed to parse JSON"),
        };

        if let Err(err) = self.parse(data) {
            error_return!("Failed to parse JSON ({})", err);
        }
    }

    fn parse(&mut self, data: &HashMap<String, JsonValue>) -> Result<()> {
        self.data.endpoint = read_value::<String>(data, "endpoint")?;
        self.data.tick = read_value::<f64>(data, "tick")? as u32;

        for server_data in read_array(data, "servers")? {
            self.data.servers.push(ConfigServerData {
                name: read_value::<String>(server_data, "name")?,
                flag: read_value::<String>(server_data, "flag")?,
                address: read_value::<String>(server_data, "address")?,
                enabled: read_value::<bool>(server_data, "enabled")?,
            });
        }

        Ok(())
    }
}
