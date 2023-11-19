pub mod client;
pub mod frame;
pub mod packet;

#[cfg(any(windows, unix))]
pub mod server;

const PING_CID: u8 = 0x00;
const PONG_CID: u8 = 0x01;
