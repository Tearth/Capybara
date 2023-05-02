#[cfg(target_arch = "x86_64")]
pub mod windows_winapi;

#[cfg(target_arch = "wasm32")]
pub mod web_wasm32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    Unknown,
    MouseMoved(i32, i32),
}
