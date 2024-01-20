use super::GlobalData;
use capybara::anyhow::Result;
use capybara::app::ApplicationState;
use capybara::assets::AssetsLoadingStatus;
use capybara::egui::Color32;
use capybara::egui::FontFamily;
use capybara::egui::FontId;
use capybara::egui::FullOutput;
use capybara::egui::RawInput;
use capybara::egui::Stroke;
use capybara::egui::TextStyle;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::InputEvent;

#[derive(Default)]
pub struct BootScene {}

impl Scene<GlobalData> for BootScene {
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
        if state.global.assets.load("./data/boot.zip") == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(&state.global.assets, None);

            let mut style = (*state.ui.inner.read().unwrap().style()).clone();
            style.text_styles = [
                (TextStyle::Heading, (FontId { size: 32.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Body, (FontId { size: 20.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Button, (FontId { size: 32.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Name("Debug".into()), (FontId { size: 20.0, family: FontFamily::Name("Kenney Pixel".into()) })),
            ]
            .into();
            style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(30, 50, 20));
            state.ui.inner.read().unwrap().set_style(style);

            return Ok(Some(FrameCommand::ChangeScene { name: "LoadingScene".to_string() }));
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        Ok((state.ui.inner.read().unwrap().run(input, |_| {}), None))
    }

    fn reset(&self) -> Box<dyn Scene<GlobalData>> {
        Box::<Self>::default()
    }
}
