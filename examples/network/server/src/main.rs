use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClient;
use capybara::network::server::listener::WebSocketListener;
use futures_channel::mpsc;
use futures_util::StreamExt;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::select;
use tokio::time;

pub mod core;

#[tokio::main]
async fn main() {
    let mut listener = WebSocketListener::new();
    let connected_clients = Arc::<RwLock<Vec<WebSocketConnectedClient>>>::new(Default::default());
    let (listener_tx, listener_rx) = mpsc::unbounded::<WebSocketConnectedClient>();
    let (packet_event_tx, mut packet_event_rx) = mpsc::unbounded::<(u64, Packet)>();
    let (disconnection_event_tx, mut disconnection_event_rx) = mpsc::unbounded::<u64>();

    let message_queue = Arc::new(RwLock::new(Vec::<(u64, Packet)>::new()));

    let connected_clients = connected_clients.clone();
    let initialize_new_clients = listener_rx.for_each(|client| async {
        connected_clients.write().unwrap().push(client);
        connected_clients.write().unwrap().last_mut().unwrap().run(packet_event_tx.clone(), disconnection_event_tx.clone());
    });

    let read_frames = async {
        while let Some((id, frame)) = packet_event_rx.next().await {
            message_queue.write().unwrap().push((id, frame));
        }
    };

    let process_disconnection = async {
        while let Some(id) = disconnection_event_rx.next().await {
            let index = connected_clients.write().unwrap().iter().position(|p| p.id == id).unwrap();
            connected_clients.write().unwrap().remove(index);
        }
    };

    let message_queue_clone = message_queue.clone();
    let process_message_queue = async {
        let mut interval = time::interval(Duration::from_millis(100));
        loop {
            while let Some(message) = message_queue_clone.write().unwrap().pop() {
                println!("Received {:?}", message);
            }

            interval.tick().await;
        }
    };
    let send_pings = async {
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            for client in connected_clients.read().unwrap().iter() {
                println!("CLIENT {}: ping {} ms", client.id, client.ping.read().unwrap());
                client.send_ping();
            }

            interval.tick().await;
        }
    };

    select! {
        _ = listener.listen("localhost:9999", listener_tx) => {}
        _ = initialize_new_clients => {}
        _ = read_frames => {}
        _ = process_disconnection => {}
        _ = process_message_queue => {}
        _ = send_pings => {}
    }
}
