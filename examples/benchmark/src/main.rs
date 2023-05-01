use orion_core::window::InputEvent;

fn main() {
    let mut window = orion_core::window::windows_winapi::Window::new("test").unwrap();
    loop {
        for event in window.poll_event() {
            if let InputEvent::MouseMoved(x, y) = event {
                println!("{} {}", x, y);
            }
        }
    }
}
