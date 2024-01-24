use crate::core::QueuePacket;
use crate::servers::ServerConnection;
use capybara::network::client::ConnectionStatus;
use capybara::network::packet::Packet;
use capybara::network::server::client::WebSocketConnectedClientSlim;
use capybara::rustc_hash::FxHashMap;
use capybara::utils::string::StringUtils;
use network_template_base::packets::*;

pub struct Lobby {}

impl Lobby {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, clients: &FxHashMap<u64, WebSocketConnectedClientSlim>, servers: &Vec<ServerConnection>, packets: Vec<QueuePacket>) {
        for packet in packets {
            match packet.inner.get_id() {
                Some(PACKET_PLAYER_NAME_REQUEST) => {
                    if let Some(client) = clients.get(&packet.client_id) {
                        client.send_packet(Packet::from_object(
                            PACKET_PLAYER_NAME_RESPONSE,
                            &PacketPlayerNameResponse { name: "Funny Fauna".as_bytes_array() },
                        ));
                    }
                }
                Some(PACKET_SERVER_LIST_REQUEST) => {
                    if let Some(client) = clients.get(&packet.client_id) {
                        let mut count = 0;
                        let mut data = [PacketServerListData::default(); 3];

                        for server in servers {
                            if server.definition.enabled && *server.websocket.status.read().unwrap() == ConnectionStatus::Connected {
                                data[count as usize] = PacketServerListData {
                                    name: server.definition.name.as_bytes_array(),
                                    flag: server.definition.flag.as_bytes_array(),
                                    address: server.definition.address.as_bytes_array(),
                                };

                                count += 1;
                            }
                        }

                        client.send_packet(Packet::from_object(PACKET_SERVER_LIST_RESPONSE, &PacketServerListResponse { count, servers: data }));
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for Lobby {
    fn default() -> Self {
        Self::new()
    }
}
