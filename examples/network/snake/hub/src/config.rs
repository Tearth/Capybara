use capybara::anyhow::Result;
use capybara::error_return;
use capybara::utils::json;
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
    pub lobby_tick: u32,
    pub worker_status_interval: u32,
    pub packet_delay_base: u32,
    pub packet_delay_variation: u32,
    pub workers: Vec<ConfigWorkerData>,
}

#[derive(Debug, Default, Clone)]
pub struct ConfigWorkerData {
    pub id: String,
    pub name: String,
    pub flag: String,
    pub address: String,
    pub enabled: bool,
}

impl ConfigLoader {
    pub fn new(path: &str) -> Self {
        let mut config = Self { path: path.to_string(), ..Default::default() };

        config.reload();
        config
    }

    pub fn reload(&mut self) {
        self.data = ConfigData::default();

        let mut file = match File::open(&self.path) {
            Ok(file) => file,
            Err(err) => error_return!("Failed to open configuration file ({})", err),
        };

        let mut buffer = Vec::default();
        if let Err(err) = file.read_to_end(&mut buffer) {
            error_return!("Failed to read configuration file ({})", err);
        }

        let content = match str::from_utf8(&buffer) {
            Ok(content) => content,
            Err(err) => error_return!("Failed to parse configuration content ({})", err),
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
        self.data.endpoint = json::read_value::<String>(data, "endpoint")?;
        self.data.lobby_tick = json::read_value::<f64>(data, "lobby_tick")? as u32;
        self.data.worker_status_interval = json::read_value::<f64>(data, "worker_status_interval")? as u32;
        self.data.packet_delay_base = json::read_value::<f64>(data, "packet_delay_base")? as u32;
        self.data.packet_delay_variation = json::read_value::<f64>(data, "packet_delay_variation")? as u32;

        for worker_data in json::read_array(data, "workers")? {
            self.data.workers.push(ConfigWorkerData {
                id: json::read_value::<String>(worker_data, "id")?,
                name: json::read_value::<String>(worker_data, "name")?,
                flag: json::read_value::<String>(worker_data, "flag")?,
                address: json::read_value::<String>(worker_data, "address")?,
                enabled: json::read_value::<bool>(worker_data, "enabled")?,
            });
        }

        Ok(())
    }
}
