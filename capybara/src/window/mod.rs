use glam::IVec2;

#[cfg(windows)]
pub mod winapi;
#[cfg(windows)]
pub type WindowContext = winapi::WindowContext;

#[cfg(unix)]
pub mod x11;
#[cfg(unix)]
pub type WindowContext = x11::WindowContext;

#[cfg(web)]
pub mod web;
#[cfg(web)]
pub type WindowContext = web::WindowContext;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowStyle {
    Window { size: IVec2 },
    Borderless,
    Fullscreen,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    WindowSizeChange { size: IVec2 },
    MouseMove { position: IVec2, modifiers: Modifiers },
    MouseEnter { position: IVec2, modifiers: Modifiers },
    MouseLeave,
    MouseButtonPress { button: MouseButton, position: IVec2, modifiers: Modifiers },
    MouseButtonRelease { button: MouseButton, position: IVec2, modifiers: Modifiers },
    MouseWheelRotated { direction: MouseWheelDirection, modifiers: Modifiers },
    KeyPress { key: Key, repeat: bool, modifiers: Modifiers },
    KeyRelease { key: Key, modifiers: Modifiers },
    CharPress { character: char, repeat: bool, modifiers: Modifiers },
    TouchStart { id: u64, position: IVec2 },
    TouchMove { id: u64, position: IVec2 },
    TouchEnd { id: u64, position: IVec2 },
    WindowClose,
    Unknown,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Key {
    Enter,
    Escape,
    Backspace,
    Space,
    Tab,
    Control,
    Shift,
    Alt,

    ArrowLeft,
    ArrowUp,
    ArrowRight,
    ArrowDown,

    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    #[default]
    Unknown,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,

    #[default]
    Unknown,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MouseWheelDirection {
    Up,
    Down,

    #[default]
    Unknown,
}

#[derive(Debug, Default)]
pub struct MemoryInfo {
    pub private: usize,
    pub reserved: usize,
}

impl Modifiers {
    pub fn new(control: bool, alt: bool, shift: bool) -> Self {
        Self { control, alt, shift }
    }
}
