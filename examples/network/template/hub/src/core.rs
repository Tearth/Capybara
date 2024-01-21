use crate::lobby::Lobby;
use crate::terminal;
use capybara::egui::ahash::HashMap;
use capybara::log::Level;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClient;
use capybara::network::server::client::WebSocketConnectedClientSlim;
use capybara::network::server::listener::WebSocketListener;
use futures_channel::mpsc;
use futures_util::StreamExt;
use network_template_base::*;
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
}

#[derive(Clone)]
pub struct QueuePacket {
    pub client_id: u64,
    pub inner: Packet,
}

impl Core {
    pub fn new() -> Self {
        Self { clients: Default::default(), queue: Default::default(), lobby: Default::default() }
    }

    pub async fn run(&mut self) {
        simple_logger::init_with_level(Level::Info).unwrap();

        let mut listener = WebSocketListener::new();
        let (listener_tx, mut listener_rx) = mpsc::unbounded::<WebSocketConnectedClient>();
        let (packet_event_tx, mut packet_event_rx) = mpsc::unbounded::<(u64, Packet)>();
        let (disconnection_event_tx, mut disconnection_event_rx) = mpsc::unbounded::<u64>();

        let clients = self.clients.clone();
        let queue = self.queue.clone();
        let lobby = self.lobby.clone();

        let listen = listener.listen("localhost:9999", listener_tx);
        let accept_clients = async {
            while let Some(mut client) = listener_rx.next().await {
                client.run(packet_event_tx.clone(), disconnection_event_tx.clone());

                lobby.write().unwrap().initialize_client(client.to_slim());
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
        let tick = async {
            let mut interval = time::interval(Duration::from_millis(TICK));
            loop {
                let clients = clients.read().unwrap().iter().map(|p| (p.1.to_slim())).collect::<Vec<WebSocketConnectedClientSlim>>();
                let packets = queue.write().unwrap().clone();

                queue.write().unwrap().clear();
                lobby.write().unwrap().tick(clients, packets);

                interval.tick().await;
            }
        };

        select! {
            _ = listen => {}
            _ = accept_clients => {}
            _ = read_frames => {}
            _ = process_disconnection => {}
            _ = process_terminal => {}
            _ = tick => {}
        }
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
