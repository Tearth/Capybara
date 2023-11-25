use capybara::glam::Vec2;

pub const TICK: u64 = 50;
pub const PACKET_OBJECTS_ARRAY: u16 = 0;
pub const PACKET_SET_VIEWPORT: u16 = 1;
pub const PACKET_SET_COUNT: u16 = 2;

#[derive(Clone)]
pub struct PacketSetViewport {
    pub size: Vec2,
}

#[derive(Clone)]
pub struct PacketSetCount {
    pub count: u32,
}
