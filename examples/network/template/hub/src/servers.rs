use capybara::network::client::{ConnectionStatus, WebSocketClient};
use log::error;

pub struct ServerManager {
    pub servers: Vec<ServerInfo>,
}

pub struct ServerInfo {
    pub name: String,
    pub flag: String,
    pub address: String,
    pub enabled: bool,
    pub websocket: WebSocketClient,
}

impl ServerManager {
    pub fn new() -> Self {
        let mut servers = vec![
            ServerInfo {
                name: "Server A".to_string(),
                flag: "EU".to_string(),
                address: "ws://localhost:10000".to_string(),
                enabled: true,
                websocket: WebSocketClient::default(),
            },
            ServerInfo {
                name: "Server B".to_string(),
                flag: "US".to_string(),
                address: "ws://localhost:10001".to_string(),
                enabled: true,
                websocket: WebSocketClient::default(),
            },
        ];

        for server in &mut servers {
            server.websocket.connect(&server.address);
        }

        Self { servers }
    }

    pub fn send_pings(&mut self) {
        for server in &mut self.servers {
            if server.enabled {
                if *server.websocket.status.read().unwrap() != ConnectionStatus::Connected {
                    error!("Server {} is disconnected, restarting", server.name);

                    server.websocket = WebSocketClient::default();
                    server.websocket.connect(&server.address);
                } else {
                    server.websocket.send_ping();
                }
            }
        }
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}
