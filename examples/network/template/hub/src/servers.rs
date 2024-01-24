use crate::config::ConfigLoader;
use crate::config::ConfigServerData;
use capybara::network::client::ConnectionStatus;
use capybara::network::client::WebSocketClient;
use log::error;

pub struct ServerManager {
    pub servers: Vec<ServerConnection>,
}

pub struct ServerConnection {
    pub definition: ConfigServerData,
    pub websocket: WebSocketClient,
}

impl ServerManager {
    pub fn new(config: &ConfigLoader) -> Self {
        let mut servers = Vec::new();

        for server_definition in &config.data.servers {
            servers.push(ServerConnection { definition: server_definition.clone(), websocket: WebSocketClient::default() })
        }

        for server in &mut servers {
            server.websocket.connect(&server.definition.address);
        }

        Self { servers }
    }

    pub fn send_pings(&mut self) {
        for server in &mut self.servers {
            if server.definition.enabled {
                if *server.websocket.status.read().unwrap() != ConnectionStatus::Connected {
                    error!("Server {} is disconnected, restarting", server.definition.name);

                    server.websocket = WebSocketClient::default();
                    server.websocket.connect(&server.definition.address);
                } else {
                    server.websocket.send_ping();
                }
            }
        }
    }
}
