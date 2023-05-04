#[cfg(windows)]
pub mod winapi;

#[cfg(unix)]
pub mod x11;

#[cfg(web)]
pub mod web;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WindowStyle {
    Window { size: Coordinates },
    Borderless,
    Fullscreen,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Unknown,
    WindowSizeChange { size: Coordinates },
    MouseMove { position: Coordinates, modifiers: Modifiers },
    MouseEnter { position: Coordinates, modifiers: Modifiers },
    MouseLeave,
    MouseButtonPress { button: MouseButton, position: Coordinates, modifiers: Modifiers },
    MouseButtonRelease { button: MouseButton, position: Coordinates, modifiers: Modifiers },
    MouseWheelRotated { direction: MouseWheelDirection, modifiers: Modifiers },
    KeyPress { key: Key, repeat: bool, modifiers: Modifiers },
    KeyRelease { key: Key, modifiers: Modifiers },
    CharPress { character: char, repeat: bool, modifiers: Modifiers },
    WindowClose,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Enter,
    Escape,
    Backspace,
    Space,
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

    Unknown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Unknown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MouseWheelDirection {
    Up,
    Down,
    Unknown,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Modifiers {
    pub fn new(control: bool, alt: bool, shift: bool) -> Self {
        Self { control, alt, shift }
    }
}

impl Coordinates {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
