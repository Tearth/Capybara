use capybara::anyhow::Result;
use capybara::app::ApplicationContext;
use capybara::app::ApplicationState;
use capybara::assets::loader::AssetsLoader;
use capybara::assets::AssetsLoadingStatus;
use capybara::egui::CentralPanel;
use capybara::egui::FullOutput;
use capybara::egui::RawInput;
use capybara::egui::ScrollArea;
use capybara::fast_gpu;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::Coordinates;
use capybara::window::InputEvent;
use capybara::window::WindowStyle;
use test::ColorTest;

pub mod test;

fast_gpu!();

#[derive(Default)]
struct GlobalData {
    assets: AssetsLoader,
}

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
        if state.global.assets.load("./data/data0.zip") == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(&state.global.assets, None);
            self.initialized = true;
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().unwrap().run(input, |context| {
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
    main_internal().unwrap();
}

fn main_internal() -> Result<()> {
    ApplicationContext::<GlobalData>::new("UI", WindowStyle::Window { size: Coordinates::new(1280, 720) })?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
