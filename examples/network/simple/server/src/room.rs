use crate::core::QueuePacket;
use capybara::error_continue;
use capybara::glam::Vec2;
use capybara::instant::Instant;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClientSlim;
use capybara::utils::rand::NewRand;
use simple_base::*;

pub struct Room {
    last_update: Option<Instant>,
    objects: Vec<ServerObject>,
    objects_count: u32,
    viewport: Vec2,
}

#[derive(Clone)]
pub struct ServerObject {
    position: Vec2,
    direction: Vec2,
}

impl Room {
    pub fn new() -> Self {
        Self { last_update: None, objects: Vec::new(), objects_count: 100, viewport: Vec2::new(512.0, 512.0) }
    }

    pub fn initialize_client(&mut self, client: WebSocketConnectedClientSlim) {
        client.send_packet(Packet::from_object(PACKET_SET_COUNT, &PacketSetCount { count: self.objects_count }));
    }

    pub fn tick(&mut self, clients: Vec<WebSocketConnectedClientSlim>, packets: Vec<QueuePacket>) {
        let now = Instant::now();
        let delta = (now - self.last_update.unwrap_or(Instant::now())).as_secs_f32();
        self.last_update = Some(now);

        for packet in packets {
            match packet.inner.get_id() {
                Some(PACKET_SET_VIEWPORT) => match packet.inner.to_object::<PacketSetViewport>() {
                    Ok(packet) => self.viewport = packet.size,
                    Err(err) => error_continue!("Failed to process packet ({})", err),
                },
                Some(PACKET_SET_COUNT) => {
                    match packet.inner.to_object::<PacketSetCount>() {
                        Ok(packet) => self.objects_count = packet.count,
                        Err(err) => error_continue!("Failed to process packet ({})", err),
                    };

                    for client in &clients {
                        if client.id != packet.client_id {
                            client.send_packet(Packet::from_object(PACKET_SET_COUNT, &PacketSetCount { count: self.objects_count }));
                        }
                    }
                }
                _ => {}
            }
        }

        if self.objects.is_empty() || self.objects.len() != self.objects_count as usize {
            self.objects.clear();

            for _ in 0..self.objects_count {
                let position = Vec2::new_rand(0.0..1.0) * self.viewport;
                self.objects.push(ServerObject { position, direction: Vec2::new_rand(-1.0..1.0) });
            }
        }

        for object in &mut self.objects {
            object.position += object.direction * 100.0 * delta;

            if object.position.x < 0.0 {
                object.direction = Vec2::new(object.direction.x.abs(), object.direction.y);
            } else if object.position.x > self.viewport.x {
                object.direction = Vec2::new(-object.direction.x.abs(), object.direction.y);
            } else if object.position.y < 0.0 {
                object.direction = Vec2::new(object.direction.x, object.direction.y.abs());
            } else if object.position.y > self.viewport.y {
                object.direction = Vec2::new(object.direction.x, -object.direction.y.abs());
            }
        }

        let positions = self.objects.iter().map(|p| p.position).collect::<Vec<Vec2>>();
        for client in clients {
            client.send_packet(Packet::from_array(PACKET_OBJECTS_ARRAY, &positions));
        }
    }
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}
