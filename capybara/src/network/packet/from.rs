use super::*;

impl From<Packet> for Vec<u8> {
    fn from(packet: Packet) -> Self {
        match packet {
            Packet::Ping { timestamp } => ping(timestamp),
            Packet::Pong { timestamp } => pong(timestamp),
            Packet::Object { id, data } => object(id, data),
            Packet::Array { id, length, data } => array(id, length, data),
            Packet::Unknown => Vec::new(),
        }
    }
}

fn ping(timestamp: u64) -> Vec<u8> {
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut output = vec![PING_CID];
    output.extend_from_slice(&timestamp_bytes);

    output
}

fn pong(timestamp: u64) -> Vec<u8> {
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut output = vec![PONG_CID];
    output.extend_from_slice(&timestamp_bytes);

    output
}

fn object(id: u16, data: Vec<u8>) -> Vec<u8> {
    let mut output = vec![OBJECT_CID];
    output.extend_from_slice(&id.to_le_bytes());
    output.extend_from_slice(&data);

    output
}

fn array(id: u16, length: u32, data: Vec<u8>) -> Vec<u8> {
    let mut output = vec![ARRAY_CID];
    output.extend_from_slice(&id.to_le_bytes());
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(&data);

    output
}
