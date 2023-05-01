#[cfg(target_os = "windows")]
pub mod windows_winapi;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    Unknown,
    MouseMoved(i32, i32),
}
