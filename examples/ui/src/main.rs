use capybara_core::anyhow::Result;
use capybara_core::app::ApplicationContext;
use capybara_core::app::ApplicationState;
use capybara_core::assets::AssetsLoadingStatus;
use capybara_core::egui::CentralPanel;
use capybara_core::egui::FullOutput;
use capybara_core::egui::RawInput;
use capybara_core::egui::ScrollArea;
use capybara_core::fast_gpu;
use capybara_core::scene::FrameCommand;
use capybara_core::scene::Scene;
use capybara_core::window::Coordinates;
use capybara_core::window::InputEvent;
use capybara_core::window::WindowStyle;
use test::ColorTest;

pub mod test;

fast_gpu!();

#[derive(Default)]
struct GlobalData {}

#[derive(Default)]
struct MainScene {
    initialized: bool,
    test: ColorTest,
}

impl Scene<GlobalData> for MainScene {
    fn activation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState<GlobalData>, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn fixed(&mut self, _: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, _: f32, _: f32) -> Result<Option<FrameCommand>> {
        if state.assets.load("./data/data0.zip")? == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(state.assets, None)?;
            self.initialized = true;
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                    if self.initialized {
                        self.test.ui(ui);
                    }
                });
            });
        });

        Ok((output, None))
    }
}

fn main() {
    ApplicationContext::<GlobalData>::new("UI", WindowStyle::Window { size: Coordinates::new(1280, 720) })
        .unwrap()
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene")
        .unwrap();
}
