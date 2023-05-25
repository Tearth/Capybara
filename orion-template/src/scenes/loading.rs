use orion_core::anyhow::Result;
use orion_core::app::ApplicationState;
use orion_core::assets::AssetsLoadingStatus;
use orion_core::egui::CentralPanel;
use orion_core::egui::Color32;
use orion_core::egui::Direction;
use orion_core::egui::FullOutput;
use orion_core::egui::Layout;
use orion_core::egui::RawInput;
use orion_core::egui::RichText;
use orion_core::instant::Instant;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::window::InputEvent;

#[derive(Default)]
pub struct LoadingScene {
    loading_start: Option<Instant>,
    loading_finished: bool,
}

impl Scene for LoadingScene {
    fn activation(&mut self, _: ApplicationState) -> Result<()> {
        self.loading_start = Some(Instant::now());
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, state: ApplicationState, _: f32) -> Result<Option<FrameCommand>> {
        if state.assets.load("./data/main.zip")? == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(state.assets, None)?;
            state.ui.instantiate_assets(state.assets, None)?;
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

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
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
