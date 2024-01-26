pub const PACKET_PLAYER_NAME_REQUEST: u16 = 0;
pub const PACKET_PLAYER_NAME_RESPONSE: u16 = 1;
pub const PACKET_SERVER_LIST_REQUEST: u16 = 2;
pub const PACKET_SERVER_LIST_RESPONSE: u16 = 3;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PacketPlayerNameRequest {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PacketPlayerNameResponse {
    pub name: [u8; 64],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PacketServerListRequest {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketServerListResponse {
    pub name: [u8; 64],
    pub flag: [u8; 4],
    pub address: [u8; 32],
}

impl Default for PacketServerListResponse {
    fn default() -> Self {
        Self { name: [0; 64], flag: [0; 4], address: [0; 32] }
    }
}
