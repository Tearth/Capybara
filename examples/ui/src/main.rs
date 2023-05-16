pub mod test;

use orion_core::app::ApplicationContext;
use orion_core::app::ApplicationState;
use orion_core::egui::Context;
use orion_core::user::UserSpace;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::WindowStyle;
use test::ColorTest;

#[no_mangle]
#[cfg(windows)]
pub static NvOptimusEnablement: i32 = 1;

#[no_mangle]
#[cfg(windows)]
pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

#[derive(Default)]
struct User {
    test: ColorTest,
}

impl UserSpace for User {
    fn input(&mut self, _: ApplicationState, _: InputEvent) {}

    fn frame(&mut self, _: ApplicationState, _: f32) {}

    fn ui(&mut self, _: ApplicationState, context: &Context) {
        orion_core::egui::CentralPanel::default().show(context, |ui| {
            orion_core::egui::ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                self.test.ui(ui);
            });
        });
    }
}

fn main() {
    ApplicationContext::new(User::default(), "UI", WindowStyle::Window { size: Coordinates::new(800, 600) }).unwrap().run().unwrap();
}
