use crate::config::ConfigLoader;
use crate::config::ConfigWorkerData;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use log::error;

pub struct WorkersManager {
    pub workers: Vec<WorkerConnection>,
}

pub struct WorkerConnection {
    pub definition: ConfigWorkerData,
    pub websocket: WebSocketClient,
}

impl WorkersManager {
    pub fn new(config: &ConfigLoader) -> Self {
        let mut workers = Vec::default();

        for worker_definition in &config.data.workers {
            workers.push(WorkerConnection { definition: worker_definition.clone(), websocket: WebSocketClient::default() })
        }

        for worker in &mut workers {
            worker.websocket.connect(&worker.definition.address);
        }

        Self { workers }
    }

    pub fn send_pings(&mut self) {
        for worker in &mut self.workers {
            if worker.definition.enabled {
                if *worker.websocket.status.read().unwrap() != ConnectionStatus::Connected {
                    error!("Worker {} is disconnected, restarting connection", worker.definition.name);

                    worker.websocket = WebSocketClient::default();
                    worker.websocket.connect(&worker.definition.address);
                } else {
                    worker.websocket.send_ping();
                }
            }
        }
    }
}
