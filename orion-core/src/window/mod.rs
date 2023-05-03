#[cfg(target_arch = "x86_64")]
pub mod winapi;

#[cfg(target_arch = "wasm32")]
pub mod web;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    Unknown,
    WindowSizeChange(Coordinates),
    MouseMove(Coordinates),
    MouseEnter(Coordinates),
    MouseLeave,
    WindowClose,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Coordinates {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
