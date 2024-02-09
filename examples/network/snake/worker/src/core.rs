use crate::config::ConfigLoader;
use crate::room::Room;
use crate::terminal;
use capybara::anyhow::Result;
use capybara::egui::ahash::HashMap;
use capybara::error_continue;
use capybara::fastrand;
use capybara::instant::Instant;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClient;
use capybara::network::server::listener::WebSocketListener;
use chrono::SecondsFormat;
use chrono::Utc;
use futures_channel::mpsc;
use futures_util::StreamExt;
use log::error;
use log::info;
use snake_base::packets::*;
use std::collections::VecDeque;
use std::fs;
use std::panic;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::join;
use tokio::select;
use tokio::time;

pub struct Core {
    pub clients: Arc<RwLock<HashMap<u64, WebSocketConnectedClient>>>,
    pub queue_incoming: Arc<RwLock<Vec<QueuePacket>>>,
    pub queue_outgoing: Arc<RwLock<Vec<QueuePacket>>>,
    pub rooms: Arc<RwLock<Vec<Arc<RwLock<Room>>>>>,
    pub config: Arc<RwLock<ConfigLoader>>,
}

#[derive(Clone, Debug)]
pub struct QueuePacket {
    pub client_id: u64,
    pub timestamp: Instant,
    pub inner: Packet,
}

impl Core {
    pub fn new() -> Self {
        let config = ConfigLoader::new("config.json");

        Self {
            clients: Default::default(),
            queue_incoming: Default::default(),
            queue_outgoing: Default::default(),
            rooms: Default::default(),
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
        let queue_incoming = self.queue_incoming.clone();
        let queue_outgoing = self.queue_outgoing.clone();
        let rooms = self.rooms.clone();
        let config = self.config.clone();

        let endpoint = config.read().unwrap().data.endpoint.clone();
        let listen = listener.listen(&endpoint, listener_tx);

        // Only one server in the template
        rooms.write().unwrap().push(Arc::new(RwLock::new(Room::new())));

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
                match frame.get_id() {
                    Some(PACKET_SERVER_TIME_REQUEST) => {
                        if let Some(client) = clients.read().unwrap().get(&id) {
                            client.send_packet(Packet::from_object(PACKET_SERVER_TIME_RESPONSE, &PacketServerTimeResponse { time: Instant::now() }));
                        } else {
                            error_continue!("Cannot reply with server time, client {} does not exists", id);
                        }
                    }
                    Some(PACKET_JOIN_ROOM_REQUEST) => {
                        if let Some(client) = clients.read().unwrap().get(&id) {
                            rooms.write().unwrap()[0].write().unwrap().add_player(client.id);
                            client.send_packet(Packet::from_object(
                                PACKET_JOIN_ROOM_RESPONSE,
                                &PacketJoinRoomResponse { player_id: id, tick: config.read().unwrap().data.worker_tick },
                            ));
                        } else {
                            error_continue!("Cannot reply for room join request, client {} does not exists", id);
                        }
                    }
                    Some(_) => {
                        queue_incoming.write().unwrap().push(QueuePacket::new(id, frame));
                    }
                    None => error_continue!("Invalid frame ID ({:?})", frame.get_id()),
                }
            }
        };
        let process_clients = async {
            let client_ping_interval = config.read().unwrap().data.client_ping_interval;
            let mut interval = time::interval(Duration::from_millis(client_ping_interval as u64));

            loop {
                for client in clients.read().unwrap().iter() {
                    client.1.send_ping();

                    let client_ping_interval = config.read().unwrap().data.client_ping_interval;
                    if interval.period().as_millis() != client_ping_interval as u128 {
                        interval = time::interval(Duration::from_millis(client_ping_interval as u64));
                        info!("Client ping interval changed to {} ms", client_ping_interval);
                    }
                }

                interval.tick().await;
            }
        };
        let process_disconnection = async {
            while let Some(id) = disconnection_event_rx.next().await {
                rooms.write().unwrap()[0].write().unwrap().remove_player(id);
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
            let worker_tick = config.read().unwrap().data.worker_tick;
            let mut interval = time::interval(Duration::from_millis(worker_tick as u64));

            loop {
                let now = Instant::now();
                let packets = Arc::new(RwLock::new(Vec::new()));
                let mut packets_to_remove = VecDeque::new();
                let mut handles = Vec::new();

                let worker_tick = config.read().unwrap().data.worker_tick;
                let delay_base = config.read().unwrap().data.packet_delay_base as i32;
                let delay_variation = config.read().unwrap().data.packet_delay_variation as i32;

                // Prepare a list of packets ready to process (if delay_base + variation = 0 then take everything from the queue)
                for (index, queue_packet) in queue_incoming.read().unwrap().iter().enumerate() {
                    let variation = fastrand::i32(-delay_variation..=delay_variation);
                    if queue_packet.timestamp + Duration::from_millis((delay_base + variation) as u64) <= now {
                        packets.write().unwrap().push(queue_packet.clone());
                        packets_to_remove.push_front(index);
                    }
                }

                // Remove processed packets from the queue
                while let Some(index) = packets_to_remove.pop_front() {
                    queue_incoming.write().unwrap().remove(index);
                }

                for room in rooms.write().unwrap().iter_mut() {
                    let room = room.clone();
                    let packets = packets.clone();
                    let clients = clients.clone();
                    let queue_outgoing = queue_outgoing.clone();
                    let config = config.clone();

                    handles.push(tokio::spawn(async move {
                        let packets = room.write().unwrap().tick(&packets.read().unwrap(), &config.read().unwrap());

                        if delay_base == 0 && delay_variation == 0 {
                            for packet in packets {
                                if let Some(client) = clients.read().unwrap().get(&packet.client_id) {
                                    client.send_packet(packet.inner.clone());
                                } else {
                                    error_continue!("Cannot send the frame, client {} does not exists", packet.client_id);
                                }
                            }
                        } else {
                            queue_outgoing.write().unwrap().extend_from_slice(&packets);
                        }
                    }));
                }

                for handle in handles {
                    if let Err(err) = join!(handle).0 {
                        error_continue!("Failed to perform a tick ({})", err);
                    };
                }

                // Send packets (if delay_base + variation = 0 then take everything from the queue)
                for (index, queue_packet) in queue_outgoing.read().unwrap().iter().enumerate() {
                    let variation = fastrand::i32(-delay_variation..=delay_variation);
                    if queue_packet.timestamp + Duration::from_millis((delay_base + variation) as u64) <= now {
                        if let Some(client) = clients.read().unwrap().get(&queue_packet.client_id) {
                            client.send_packet(queue_packet.inner.clone());
                        } else {
                            error!("Cannot send the frame, client {} does not exists", queue_packet.client_id);
                        }

                        packets_to_remove.push_front(index);
                    }
                }

                // Remove sent packets from the queue
                for index in &packets_to_remove {
                    queue_outgoing.write().unwrap().remove(*index);
                }

                if interval.period().as_millis() != worker_tick as u128 {
                    for client in clients.read().unwrap().values() {
                        client.send_packet(Packet::from_object(PACKET_SET_TICK_INTERVAL, &PacketSetTickInterval { tick: worker_tick }));
                    }

                    interval = time::interval(Duration::from_millis(worker_tick as u64));
                    info!("Worker tick changed to {} ms", worker_tick);
                }

                interval.tick().await;
            }
        };

        select! {
            _ = listen => {}
            _ = accept_clients => {}
            _ = read_frames => {}
            _ = process_clients => {}
            _ = process_disconnection => {}
            _ = process_terminal => {}
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

        panic::set_hook(Box::new(move |info| {
            log::error!("Critical error: {}", info);
        }));

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
