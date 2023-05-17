pub mod test;

use orion_core::anyhow::Result;
use orion_core::app::ApplicationContext;
use orion_core::app::ApplicationState;
use orion_core::egui::CentralPanel;
use orion_core::egui::FullOutput;
use orion_core::egui::RawInput;
use orion_core::egui::ScrollArea;
use orion_core::fast_gpu;
use orion_core::user::UserSpace;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::WindowStyle;
use test::ColorTest;

fast_gpu!();

#[derive(Default)]
struct User {
    test: ColorTest,
}

impl UserSpace for User {
    fn input(&mut self, _: ApplicationState, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, _: ApplicationState, _: f32) -> Result<()> {
        Ok(())
    }

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<FullOutput> {
        Ok(state.ui.inner.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                    self.test.ui(ui);
                });
            });
        }))
    }
}

fn main() {
    ApplicationContext::new(User::default(), "UI", WindowStyle::Window { size: Coordinates::new(800, 600) }).unwrap().run().unwrap();
}
