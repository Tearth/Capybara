use crate::room::Room;
use capybara::egui::ahash::HashMap;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClient;
use capybara::network::server::client::WebSocketConnectedClientSlim;
use capybara::network::server::listener::WebSocketListener;
use futures_channel::mpsc;
use futures_util::StreamExt;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::select;
use tokio::time;

pub struct Core {
    clients: Arc<RwLock<HashMap<u64, WebSocketConnectedClient>>>,
    queue: Arc<RwLock<Vec<QueuePacket>>>,
    room: Arc<RwLock<Room>>,
}

#[derive(Clone)]
pub struct QueuePacket {
    pub id: u64,
    pub packet: Packet,
}

impl Core {
    pub fn new() -> Self {
        Self { clients: Default::default(), queue: Default::default(), room: Default::default() }
    }

    pub async fn run(&mut self) {
        let mut listener = WebSocketListener::new();
        let (listener_tx, mut listener_rx) = mpsc::unbounded::<WebSocketConnectedClient>();
        let (packet_event_tx, mut packet_event_rx) = mpsc::unbounded::<(u64, Packet)>();
        let (disconnection_event_tx, mut disconnection_event_rx) = mpsc::unbounded::<u64>();

        let clients = self.clients.clone();
        let queue = self.queue.clone();
        let room = self.room.clone();

        let listen = listener.listen("localhost:9999", listener_tx);
        let initialize_new_clients = async {
            while let Some(client) = listener_rx.next().await {
                if let Some(mut client) = clients.write().unwrap().insert(client.id, client) {
                    client.run(packet_event_tx.clone(), disconnection_event_tx.clone());
                }
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
        let send_pings = async {
            let mut interval = time::interval(Duration::from_millis(1000));
            loop {
                for client in clients.read().unwrap().values() {
                    client.send_ping();
                }

                interval.tick().await;
            }
        };
        let tick = async {
            let mut interval = time::interval(Duration::from_millis(20));
            loop {
                let clients = clients.read().unwrap().iter().map(|p| (p.1.to_slim())).collect::<Vec<WebSocketConnectedClientSlim>>();
                let packets = queue.write().unwrap().clone();

                queue.write().unwrap().clear();
                room.write().unwrap().tick(clients, packets);

                interval.tick().await;
            }
        };

        select! {
            _ = listen => {}
            _ = initialize_new_clients => {}
            _ = read_frames => {}
            _ = process_disconnection => {}
            _ = send_pings => {}
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
        Self { id, packet }
    }
}
