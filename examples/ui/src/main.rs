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
use capybara::glam::IVec2;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
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
    fn activation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _state: ApplicationState<GlobalData>, _event: InputEvent) -> Result<()> {
        Ok(())
    }

    fn fixed(&mut self, _state: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, _accumulator: f32, _delta: f32) -> Result<Option<FrameCommand>> {
        if !self.initialized && state.global.assets.load("./data/data0.zip") == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(&state.global.assets, None);
            self.initialized = true;
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().run(input, |context| {
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

    fn reset(&self) -> Box<dyn Scene<GlobalData>> {
        Box::<Self>::default()
    }
}

fn main() {
    main_internal().unwrap();
}

fn main_internal() -> Result<()> {
    ApplicationContext::<GlobalData>::new("UI", WindowStyle::Window { size: IVec2::new(1280, 720) }, Some(4))?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
