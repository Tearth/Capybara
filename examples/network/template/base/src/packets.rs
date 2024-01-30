use capybara::glam::Vec2;
use capybara::instant::Instant;

pub const PACKET_PLAYER_NAME_REQUEST: u16 = 0;
pub const PACKET_PLAYER_NAME_RESPONSE: u16 = 1;
pub const PACKET_SERVER_LIST_REQUEST: u16 = 2;
pub const PACKET_SERVER_LIST_RESPONSE: u16 = 3;
pub const PACKET_SERVER_TIME_REQUEST: u16 = 4;
pub const PACKET_SERVER_TIME_RESPONSE: u16 = 5;
pub const PACKET_JOIN_ROOM_REQUEST: u16 = 6;
pub const PACKET_JOIN_ROOM_RESPONSE: u16 = 7;
pub const PACKET_PLAYER_INPUT: u16 = 8;
pub const PACKET_TICK: u16 = 9;

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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PacketServerTimeRequest {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketServerTimeResponse {
    pub time: Instant,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PacketJoinRoomRequest {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketJoinRoomResponse {
    pub player_id: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketPlayerInput {
    pub timestamp: Instant,
    pub heading: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketTickHeader {
    pub timestamp: Instant,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketTickData {
    pub player_id: u64,
    pub heading: f32,
    pub nodes: [Vec2; 5],
}

impl Default for PacketServerListResponse {
    fn default() -> Self {
        Self { name: [0; 64], flag: [0; 4], address: [0; 32] }
    }
}
