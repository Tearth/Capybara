use crate::config::ConfigLoader;
use crate::lobby::Lobby;
use crate::terminal;
use crate::workers::WorkersManager;
use capybara::anyhow::Result;
use capybara::egui::ahash::HashMap;
use capybara::error_continue;
use capybara::fastrand;
use capybara::instant::Instant;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClient;
use capybara::network::server::listener::WebSocketListener;
use capybara::parking_lot::RwLock;
use chrono::SecondsFormat;
use chrono::Utc;
use futures_channel::mpsc;
use futures_util::StreamExt;
use log::info;
use std::collections::VecDeque;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::select;
use tokio::time;

pub struct Core {
    pub clients: Arc<RwLock<HashMap<u64, WebSocketConnectedClient>>>,
    pub queue_incoming: Arc<RwLock<Vec<QueuePacket>>>,
    pub queue_outgoing: Arc<RwLock<Vec<QueuePacket>>>,
    pub lobby: Arc<RwLock<Lobby>>,
    pub workers: Arc<RwLock<WorkersManager>>,
    pub config: Arc<RwLock<ConfigLoader>>,
}

#[derive(Clone)]
pub struct QueuePacket {
    pub client_id: u64,
    pub timestamp: Instant,
    pub inner: Packet,
}

impl Core {
    pub fn new() -> Self {
        let config = ConfigLoader::new("config.json");

        Self {
            clients: Arc::default(),
            queue_incoming: Arc::default(),
            queue_outgoing: Arc::default(),
            lobby: Arc::default(),
            workers: Arc::new(RwLock::new(WorkersManager::new(&config))),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn run(&mut self) {
        if let Err(err) = self.init_logger() {
            println!("Failed to initialize logger ({})", err);
            return;
        }

        let mut listener = WebSocketListener::default();
        let (listener_tx, mut listener_rx) = mpsc::unbounded::<WebSocketConnectedClient>();
        let (packet_event_tx, mut packet_event_rx) = mpsc::unbounded::<(u64, Packet)>();
        let (disconnection_event_tx, mut disconnection_event_rx) = mpsc::unbounded::<u64>();

        let clients = self.clients.clone();
        let queue_incoming = self.queue_incoming.clone();
        let queue_outgoing = self.queue_outgoing.clone();
        let lobby = self.lobby.clone();
        let workers = self.workers.clone();
        let config = self.config.clone();

        let endpoint = config.read().data.endpoint.clone();
        let listen = listener.listen(&endpoint, listener_tx);

        let accept_clients = async {
            while let Some(mut client) = listener_rx.next().await {
                if let Err(err) = client.run(packet_event_tx.clone(), disconnection_event_tx.clone()) {
                    error_continue!("Failed to run client runtime ({})", err);
                }

                clients.write().insert(client.id, client);
            }
        };
        let read_frames = async {
            while let Some((id, frame)) = packet_event_rx.next().await {
                queue_incoming.write().push(QueuePacket::new(id, frame));
            }
        };
        let process_disconnection = async {
            while let Some(id) = disconnection_event_rx.next().await {
                clients.write().remove(&id);
            }
        };
        let process_workers = async {
            let worker_status_interval = config.read().data.worker_status_interval;
            let mut interval = time::interval(Duration::from_millis(worker_status_interval as u64));

            loop {
                let worker_statu_interval = config.read().data.worker_status_interval;

                if interval.period().as_millis() != worker_statu_interval as u128 {
                    interval = time::interval(Duration::from_millis(worker_statu_interval as u64));
                    info!("Worker status interval changed to {} ms", worker_statu_interval);
                }

                workers.write().send_pings();
                interval.tick().await;
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
            let lobby_tick = config.read().data.lobby_tick;
            let mut interval = time::interval(Duration::from_millis(lobby_tick as u64));

            loop {
                let now = Instant::now();
                let mut packets = Vec::default();
                let mut packets_to_remove = VecDeque::default();

                let lobby_tick = config.read().data.lobby_tick;
                let delay_base = config.read().data.packet_delay_base as i32;
                let delay_variation = config.read().data.packet_delay_variation as i32;

                // Prepare a list of packets ready to process (if delay_base + variation = 0 then take everything from the queue)
                for (index, queue_packet) in queue_incoming.read().iter().enumerate() {
                    let variation = fastrand::i32(-delay_variation..=delay_variation);
                    if queue_packet.timestamp + Duration::from_millis((delay_base + variation) as u64) <= now {
                        packets.push(queue_packet.clone());
                        packets_to_remove.push_front(index);
                    }
                }

                // Remove processed packets from the queue
                while let Some(index) = packets_to_remove.pop_front() {
                    queue_incoming.write().remove(index);
                }

                let outgoing_packets = lobby.write().tick(&workers.read().workers, &packets);
                queue_outgoing.write().extend_from_slice(&outgoing_packets);

                // Send packets (if delay_base + variation = 0 then take everything from the queue)
                for (index, queue_packet) in queue_outgoing.read().iter().enumerate() {
                    let variation = fastrand::i32(-delay_variation..=delay_variation);
                    if queue_packet.timestamp + Duration::from_millis((delay_base + variation) as u64) <= now {
                        if let Some(client) = clients.read().get(&queue_packet.client_id) {
                            client.send_packet(queue_packet.inner.clone());
                        }

                        packets_to_remove.push_front(index);
                    }
                }

                // Remove sent packets from the queue
                for index in &packets_to_remove {
                    queue_outgoing.write().remove(*index);
                }

                if interval.period().as_millis() != lobby_tick as u128 {
                    interval = time::interval(Duration::from_millis(lobby_tick as u64));
                    info!("Lobby tick changed to {} ms", lobby_tick);
                }

                interval.tick().await;
            }
        };

        select! {
            _ = listen => {}
            _ = accept_clients => {}
            _ = read_frames => {}
            _ = process_disconnection => {}
            _ = process_workers => {}
            _ = process_terminal => {}
            _ = tick => {}
        }
    }

    fn init_logger(&self) -> Result<()> {
        fs::create_dir_all("./logs/")?;

        fern::Dispatch::default()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] [{}] {}",
                    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(fern::Dispatch::default().level(log::LevelFilter::Debug).chain(fern::DateBased::new("./logs/", "log_info_%Y-%m-%d.log")))
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
        Self { client_id: id, timestamp: Instant::now(), inner: packet }
    }
}
