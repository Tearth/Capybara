use super::frame::Frame;
use super::*;

#[derive(Debug)]
pub enum Packet {
    Ping { timestamp: u128 },
    Pong { timestamp: u128 },
    Unknown,
}

impl From<Packet> for Frame {
    fn from(packet: Packet) -> Self {
        match packet {
            Packet::Ping { timestamp } => ping(timestamp),
            Packet::Pong { timestamp } => pong(timestamp),
            Packet::Unknown => Frame::Unknown,
        }
    }
}

fn ping(timestamp: u128) -> Frame {
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut data = vec![PING_CID];
    data.extend_from_slice(&timestamp_bytes);

    Frame::new_binary(data)
}

fn pong(timestamp: u128) -> Frame {
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut data = vec![PONG_CID];
    data.extend_from_slice(&timestamp_bytes);

    Frame::new_binary(data)
}
