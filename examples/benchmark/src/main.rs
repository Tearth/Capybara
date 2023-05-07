use orion_core::anyhow::Result;
use orion_core::app::ApplicationContext;
use orion_core::app::ApplicationState;
use orion_core::egui::CentralPanel;
use orion_core::egui::Color32;
use orion_core::egui::Context;
use orion_core::egui::RichText;
use orion_core::log::debug;
use orion_core::user::UserSpace;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::Key;
use orion_core::window::WindowStyle;

#[derive(Default)]
struct User {}

impl UserSpace for User {
    fn input(&mut self, state: ApplicationState, event: InputEvent) {
        debug!("New event: {:?}", event);

        if let InputEvent::KeyPress { key, repeat: _, modifiers: _ } = event {
            if key == Key::Escape {
                state.window.close();
            } else if key == Key::Space {
                state.window.set_cursor_visibility(!state.window.cursor_visible);
            }
        }
    }

    fn frame(&mut self, _: ApplicationState, delta: f32) {
        debug!("Delta: {:?}", 1.0 / delta);
    }

    fn ui(&mut self, state: ApplicationState, context: &Context) {
        CentralPanel::default().show(context, |ui| {
            ui.label(RichText::new(format!("FPS: {}", state.renderer.fps)).heading().color(Color32::from_rgb(255, 255, 255)));
        });
    }
}

fn main() -> Result<()> {
    ApplicationContext::new(User::default(), "Benchmark", WindowStyle::Window { size: Coordinates::new(800, 600) })?.run()
}
