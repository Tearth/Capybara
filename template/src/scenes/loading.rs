use super::GlobalData;
use capybara::anyhow::Result;
use capybara::app::ApplicationState;
use capybara::assets::AssetsLoadingStatus;
use capybara::egui::CentralPanel;
use capybara::egui::Color32;
use capybara::egui::Direction;
use capybara::egui::FullOutput;
use capybara::egui::Layout;
use capybara::egui::RawInput;
use capybara::egui::RichText;
use capybara::instant::Instant;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::InputEvent;

#[derive(Default)]
pub struct LoadingScene {
    loading_start: Option<Instant>,
    loading_finished: bool,
}

impl Scene<GlobalData> for LoadingScene {
    fn activation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        self.loading_start = Some(Instant::now());
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
        if state.global.assets.load("./data/main.zip")? == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(&state.global.assets, None)?;
            state.ui.instantiate_assets(&state.global.assets, None)?;
            self.loading_finished = true;
        }

        if self.loading_finished {
            if let Some(loading_start) = self.loading_start {
                #[allow(unused_comparisons, clippy::absurd_extreme_comparisons)]
                if loading_start.elapsed().as_secs() >= 0 {
                    return Ok(Some(FrameCommand::ChangeScene { name: "MenuScene".to_string() }));
                }
            }
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                    ui.label(RichText::new("Loading...".to_string()).heading().color(Color32::from_rgb(255, 255, 255)));
                });
            });
        });

        Ok((output, None))
    }
}
