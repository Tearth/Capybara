use orion_core::anyhow::Result;
use orion_core::app::ApplicationContext;
use orion_core::app::ApplicationState;
use orion_core::egui::CentralPanel;
use orion_core::egui::Color32;
use orion_core::egui::Context;
use orion_core::egui::RichText;
use orion_core::glam::Vec2;
use orion_core::log::debug;
use orion_core::renderer::sprite::Sprite;
use orion_core::user::UserSpace;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::Key;
use orion_core::window::WindowStyle;

#[no_mangle]
#[cfg(not(debug_assertions))]
pub static NvOptimusEnablement: i32 = 1;

#[no_mangle]
#[cfg(not(debug_assertions))]
pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

#[derive(Default)]
struct User {
    objects: Vec<Object>,
    initialized: bool,
}

struct Object {
    sprite: Sprite,
    direction: Vec2,
}

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

    fn frame(&mut self, state: ApplicationState, delta: f32) {
        if !self.initialized {
            state.renderer.set_viewport(Vec2::new(800.0, 600.0));
            for _ in 0..200000 {
                let position = Vec2::new(fastrand::u32(0..state.renderer.viewport_size.x as u32) as f32, fastrand::u32(0..state.renderer.viewport_size.y as u32) as f32);
                let direction = Vec2::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0);
                let sprite = Sprite { position, rotation: 0.0, scale: Vec2::new(1.0, 1.0), size: Vec2::new(10.0, 10.0) };

                self.objects.push(Object { sprite, direction });
            }

            self.initialized = true;
        }

        const SPEED: f32 = 100.0;

        for object in &mut self.objects {
            object.sprite.position += object.direction * SPEED * delta;
            if object.sprite.position.x < 0.0 {
                object.direction = Vec2::new(object.direction.x.abs(), object.direction.y);
            } else if object.sprite.position.x > state.renderer.viewport_size.x {
                object.direction = Vec2::new(-object.direction.x.abs(), object.direction.y);
            } else if object.sprite.position.y < 0.0 {
                object.direction = Vec2::new(object.direction.x, object.direction.y.abs());
            } else if object.sprite.position.y > state.renderer.viewport_size.y {
                object.direction = Vec2::new(object.direction.x, -object.direction.y.abs());
            }

            state.renderer.draw(&object.sprite);
        }

        //debug!("Delta: {:?}", 1.0 / delta);
    }

    fn ui(&mut self, state: ApplicationState, context: &Context) {
        orion_core::egui::SidePanel::new(orion_core::egui::panel::Side::Left, orion_core::egui::Id::new("test")).show(context, |ui| {
            ui.label(RichText::new(format!("FPS: {}", state.renderer.fps)).heading().color(Color32::from_rgb(255, 255, 255)));
        });
    }
}

fn main() {
    ApplicationContext::new(User::default(), "Benchmark", WindowStyle::Window { size: Coordinates::new(800, 600) }).unwrap().run().unwrap();
}
