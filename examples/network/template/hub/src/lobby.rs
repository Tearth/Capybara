use crate::core::QueuePacket;
use crate::workers::WorkerConnection;
use capybara::network::client::ConnectionStatus;
use capybara::network::packet::Packet;
use capybara::utils::string::StringUtils;
use network_template_base::packets::*;

pub struct Lobby {}

impl Lobby {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, workers: &Vec<WorkerConnection>, packets: Vec<QueuePacket>) -> Vec<QueuePacket> {
        let mut outgoing_packets = Vec::new();

        for packet in packets {
            match packet.inner.get_id() {
                Some(PACKET_PLAYER_NAME_REQUEST) => {
                    outgoing_packets.push(QueuePacket::new(
                        packet.client_id,
                        Packet::from_object(PACKET_PLAYER_NAME_RESPONSE, &PacketPlayerNameResponse { name: "Funny Fauna".as_bytes_array() }),
                    ));
                }
                Some(PACKET_SERVER_LIST_REQUEST) => {
                    let mut count = 0;
                    let mut data = [PacketServerListData::default(); 3];

                    for worker in workers {
                        if worker.definition.enabled && *worker.websocket.status.read().unwrap() == ConnectionStatus::Connected {
                            data[count as usize] = PacketServerListData {
                                id: worker.definition.id.as_bytes_array(),
                                name: worker.definition.name.as_bytes_array(),
                                flag: worker.definition.flag.as_bytes_array(),
                                address: worker.definition.address.as_bytes_array(),
                            };

                            count += 1;
                        }
                    }

                    outgoing_packets.push(QueuePacket::new(
                        packet.client_id,
                        Packet::from_object(PACKET_SERVER_LIST_RESPONSE, &PacketServerListResponse { count, servers: data }),
                    ));
                }
                _ => {}
            }
        }

        outgoing_packets
    }
}

impl Default for Lobby {
    fn default() -> Self {
        Self::new()
    }
}
