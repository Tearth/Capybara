use super::*;

impl From<Vec<u8>> for Packet {
    fn from(data: Vec<u8>) -> Self {
        match data.first() {
            Some(cid) => match *cid {
                PING_CID => ping(&data),
                PONG_CID => pong(&data),
                OBJECT_CID => object(&data),
                ARRAY_CID => array(&data),
                _ => Packet::Unknown,
            },
            None => Packet::Unknown,
        }
    }
}

fn ping(data: &[u8]) -> Packet {
    // 1b CID + 8b timestamp
    if data.len() != 9 {
        return Packet::Unknown;
    }

    let mut timestamp_bytes = [0; 8];
    timestamp_bytes.copy_from_slice(&data[1..]);

    Packet::Ping { timestamp: u64::from_le_bytes(timestamp_bytes) }
}

fn pong(data: &[u8]) -> Packet {
    // 1b CID + 8b timestamp
    if data.len() != 9 {
        return Packet::Unknown;
    }

    let mut timestamp_bytes = [0; 8];
    timestamp_bytes.copy_from_slice(&data[1..]);

    Packet::Pong { timestamp: u64::from_le_bytes(timestamp_bytes) }
}

fn object(data: &[u8]) -> Packet {
    // 1b CID + 2b OID
    if data.len() < 3 {
        return Packet::Unknown;
    }

    let mut oid_bytes = [0; 2];
    oid_bytes.copy_from_slice(&data[1..=2]);

    Packet::Object { oid: u16::from_le_bytes(oid_bytes), data: data[3..].to_vec() }
}

fn array(data: &[u8]) -> Packet {
    // 1b CID + 2b AID + 4b size
    if data.len() < 7 {
        return Packet::Unknown;
    }

    let mut aid_bytes = [0; 2];
    aid_bytes.copy_from_slice(&data[1..=2]);

    let mut length_bytes = [0; 4];
    length_bytes.copy_from_slice(&data[3..=6]);

    Packet::Array { aid: u16::from_le_bytes(aid_bytes), length: u32::from_le_bytes(length_bytes), data: data[7..].to_vec() }
}
