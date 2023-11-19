use super::packet::Packet;
use super::{PING_CID, PONG_CID};

#[derive(Debug, Clone)]
pub enum Frame {
    Text { text: String },
    Binary { data: Vec<u8> },
    Unknown,
}

impl Frame {
    pub fn new_text(text: String) -> Self {
        Frame::Text { text }
    }

    pub fn new_binary(data: Vec<u8>) -> Self {
        Frame::Binary { data }
    }
}

impl From<Frame> for Packet {
    fn from(frame: Frame) -> Self {
        if let Frame::Binary { data } = frame {
            match data.first() {
                Some(cid) => match *cid {
                    PING_CID => ping(&data),
                    PONG_CID => pong(&data),
                    _ => Packet::Unknown,
                },
                None => Packet::Unknown,
            }
        } else {
            Packet::Unknown
        }
    }
}

fn ping(data: &[u8]) -> Packet {
    // 1b CID + 16b timestamp
    if data.len() != 17 {
        return Packet::Unknown;
    }

    let mut timestamp_bytes = [0; 16];
    timestamp_bytes.copy_from_slice(&data[1..]);

    Packet::Ping { timestamp: u128::from_le_bytes(timestamp_bytes) }
}

fn pong(data: &[u8]) -> Packet {
    // 1b CID + 16b timestamp
    if data.len() != 17 {
        return Packet::Unknown;
    }

    let mut timestamp_bytes = [0; 16];
    timestamp_bytes.copy_from_slice(&data[1..]);

    Packet::Pong { timestamp: u128::from_le_bytes(timestamp_bytes) }
}
