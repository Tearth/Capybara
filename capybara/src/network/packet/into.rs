use super::*;

impl From<Vec<u8>> for Packet {
    fn from(data: Vec<u8>) -> Self {
        match data.first() {
            Some(cid) => match *cid {
                PING_CID => ping(&data),
                PONG_CID => pong(&data),
                _ => Packet::Unknown,
            },
            None => Packet::Unknown,
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
