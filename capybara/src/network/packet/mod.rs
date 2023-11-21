pub mod from;
pub mod into;

const PING_CID: u8 = 0x00;
const PONG_CID: u8 = 0x01;

#[derive(Debug)]
pub enum Packet {
    Ping { timestamp: u128 },
    Pong { timestamp: u128 },
    Unknown,
}
