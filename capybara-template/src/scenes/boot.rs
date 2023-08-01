use super::GlobalData;
use capybara_core::anyhow::Result;
use capybara_core::app::ApplicationState;
use capybara_core::assets::AssetsLoadingStatus;
use capybara_core::egui::Color32;
use capybara_core::egui::FontFamily;
use capybara_core::egui::FontId;
use capybara_core::egui::FullOutput;
use capybara_core::egui::RawInput;
use capybara_core::egui::Stroke;
use capybara_core::egui::TextStyle;
use capybara_core::scene::FrameCommand;
use capybara_core::scene::Scene;
use capybara_core::window::InputEvent;

#[derive(Default)]
pub struct BootScene {}

impl Scene<GlobalData> for BootScene {
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
        if state.assets.load("./data/boot.zip")? == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(state.assets, None)?;

            let mut style = (*state.ui.inner.style()).clone();
            style.text_styles = [
                (TextStyle::Heading, (FontId { size: 32.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Body, (FontId { size: 20.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Button, (FontId { size: 32.0, family: FontFamily::Name("Kenney Pixel".into()) })),
            ]
            .into();
            style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(30, 50, 20));
            state.ui.inner.set_style(style);

            return Ok(Some(FrameCommand::ChangeScene { name: "LoadingScene".to_string() }));
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        Ok((state.ui.inner.run(input, |_| {}), None))
    }
}
