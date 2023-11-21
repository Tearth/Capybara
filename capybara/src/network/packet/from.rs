use super::*;

impl From<Packet> for Vec<u8> {
    fn from(packet: Packet) -> Self {
        match packet {
            Packet::Ping { timestamp } => ping(timestamp),
            Packet::Pong { timestamp } => pong(timestamp),
            Packet::Unknown => Vec::new(),
        }
    }
}

fn ping(timestamp: u128) -> Vec<u8> {
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut data = vec![PING_CID];
    data.extend_from_slice(&timestamp_bytes);

    data
}

fn pong(timestamp: u128) -> Vec<u8> {
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut data = vec![PONG_CID];
    data.extend_from_slice(&timestamp_bytes);

    data
}
