pub const PACKET_PLAYER_NAME_REQUEST: u16 = 0;
pub const PACKET_PLAYER_NAME_RESPONSE: u16 = 1;
pub const PACKET_SERVER_LIST_REQUEST: u16 = 2;
pub const PACKET_SERVER_LIST_RESPONSE: u16 = 3;

#[derive(Debug, Clone)]
pub struct PacketPlayerNameRequest {}

#[derive(Debug, Clone)]
pub struct PacketPlayerNameResponse {
    pub name: [u8; 64],
}

#[derive(Debug, Clone)]
pub struct PacketServerListRequest {}

#[derive(Debug, Clone)]
pub struct PacketServerListResponse {
    pub servers: [PacketServerListData; 4],
}

#[derive(Debug, Clone)]
pub struct PacketServerListData {
    pub name: [u8; 64],
    pub flag: [u8; 4],
    pub address: [u8; 32],
}
