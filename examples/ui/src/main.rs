use orion_core::anyhow::Result;
use orion_core::app::ApplicationContext;
use orion_core::app::ApplicationState;
use orion_core::egui::CentralPanel;
use orion_core::egui::FullOutput;
use orion_core::egui::RawInput;
use orion_core::egui::ScrollArea;
use orion_core::fast_gpu;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::WindowStyle;
use test::ColorTest;

pub mod test;

fast_gpu!();

#[derive(Default)]
struct MainScene {
    test: ColorTest,
}

impl Scene for MainScene {
    fn activation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, _: ApplicationState, _: f32) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                    self.test.ui(ui);
                });
            });
        });

        Ok((output, None))
    }
}

fn main() {
    ApplicationContext::new("UI", WindowStyle::Window { size: Coordinates::new(800, 600) })
        .unwrap()
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene")
        .unwrap();
}
