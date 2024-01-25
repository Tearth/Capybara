use crate::config::ConfigLoader;
use crate::lobby::Lobby;
use crate::servers::ServerManager;
use crate::terminal;
use capybara::anyhow::Result;
use capybara::egui::ahash::HashMap;
use capybara::error_continue;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClient;
use capybara::network::server::listener::WebSocketListener;
use capybara::rustc_hash::FxHashMap;
use chrono::SecondsFormat;
use chrono::Utc;
use futures_channel::mpsc;
use futures_util::StreamExt;
use log::info;
use std::fs;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::select;
use tokio::time;

pub struct Core {
    pub clients: Arc<RwLock<HashMap<u64, WebSocketConnectedClient>>>,
    pub queue: Arc<RwLock<Vec<QueuePacket>>>,
    pub lobby: Arc<RwLock<Lobby>>,
    pub servers: Arc<RwLock<ServerManager>>,
    pub config: Arc<RwLock<ConfigLoader>>,
}

#[derive(Clone)]
pub struct QueuePacket {
    pub client_id: u64,
    pub inner: Packet,
}

impl Core {
    pub fn new() -> Self {
        let config = ConfigLoader::new("config.json");

        Self {
            clients: Default::default(),
            queue: Default::default(),
            lobby: Default::default(),
            servers: Arc::new(RwLock::new(ServerManager::new(&config))),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn run(&mut self) {
        if let Err(err) = self.init_logger() {
            println!("Failed to initialize logger ({})", err);
            return;
        }

        let mut listener = WebSocketListener::new();
        let (listener_tx, mut listener_rx) = mpsc::unbounded::<WebSocketConnectedClient>();
        let (packet_event_tx, mut packet_event_rx) = mpsc::unbounded::<(u64, Packet)>();
        let (disconnection_event_tx, mut disconnection_event_rx) = mpsc::unbounded::<u64>();

        let clients = self.clients.clone();
        let queue = self.queue.clone();
        let lobby = self.lobby.clone();
        let servers = self.servers.clone();
        let config = self.config.clone();

        let endpoint = config.read().unwrap().data.endpoint.clone();
        let listen = listener.listen(&endpoint, listener_tx);

        let accept_clients = async {
            while let Some(mut client) = listener_rx.next().await {
                if let Err(err) = client.run(packet_event_tx.clone(), disconnection_event_tx.clone()) {
                    error_continue!("Failed to run client runtime ({})", err);
                }

                clients.write().unwrap().insert(client.id, client);
            }
        };
        let read_frames = async {
            while let Some((id, frame)) = packet_event_rx.next().await {
                queue.write().unwrap().push(QueuePacket::new(id, frame));
            }
        };
        let process_disconnection = async {
            while let Some(id) = disconnection_event_rx.next().await {
                clients.write().unwrap().remove(&id);
            }
        };
        let process_terminal = async {
            let mut stdin = tokio::io::stdin();
            loop {
                let mut buffer = vec![0; 128];
                let n = match stdin.read(&mut buffer).await {
                    Err(_) | Ok(0) => break,
                    Ok(n) => n,
                };
                buffer.truncate(n);

                let command = match String::from_utf8(buffer) {
                    Ok(command) => command,
                    Err(_) => break,
                };

                terminal::process(&command, self);
            }
        };
        let process_servers = async {
            let server_status_interval = config.read().unwrap().data.server_status_interval;
            let mut interval = time::interval(Duration::from_millis(server_status_interval as u64));

            loop {
                let server_statu_interval = config.read().unwrap().data.server_status_interval;

                if interval.period().as_millis() != server_statu_interval as u128 {
                    interval = time::interval(Duration::from_millis(server_statu_interval as u64));
                    info!("Server status interval changed to {} ms", server_statu_interval);
                }

                servers.write().unwrap().send_pings();
                interval.tick().await;
            }
        };
        let tick = async {
            let lobby_tick = config.read().unwrap().data.lobby_tick;
            let mut interval = time::interval(Duration::from_millis(lobby_tick as u64));

            loop {
                let packets = queue.write().unwrap().clone();
                let clients = clients.read().unwrap().iter().map(|(id, client)| (*id, client.to_slim())).collect::<FxHashMap<_, _>>();
                let lobby_tick = config.read().unwrap().data.lobby_tick;

                if interval.period().as_millis() != lobby_tick as u128 {
                    interval = time::interval(Duration::from_millis(lobby_tick as u64));
                    info!("Lobby tick changed to {} ms", lobby_tick);
                }

                queue.write().unwrap().clear();
                lobby.write().unwrap().tick(&clients, &servers.read().unwrap().servers, packets);

                interval.tick().await;
            }
        };

        select! {
            _ = listen => {}
            _ = accept_clients => {}
            _ = read_frames => {}
            _ = process_disconnection => {}
            _ = process_terminal => {}
            _ = process_servers => {}
            _ = tick => {}
        }
    }

    fn init_logger(&self) -> Result<()> {
        fs::create_dir_all("./logs/")?;

        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] [{}] {}",
                    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(fern::Dispatch::new().level(log::LevelFilter::Debug).chain(fern::DateBased::new("./logs/", "log_info_%Y-%m-%d.log")))
            .apply()?;

        Ok(())
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl QueuePacket {
    pub fn new(id: u64, packet: Packet) -> Self {
        Self { client_id: id, inner: packet }
    }
}
