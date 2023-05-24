use orion_core::anyhow::Result;
use orion_core::app::ApplicationState;
use orion_core::assets::AssetsLoadingStatus;
use orion_core::egui::FontFamily;
use orion_core::egui::FontId;
use orion_core::egui::FullOutput;
use orion_core::egui::RawInput;
use orion_core::egui::TextStyle;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::window::InputEvent;

#[derive(Default)]
pub struct BootScene {}

impl Scene for BootScene {
    fn activation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, state: ApplicationState, _: f32) -> Result<Option<FrameCommand>> {
        if state.assets.load("./data/boot.zip")? == AssetsLoadingStatus::Finished {
            state.ui.instantiate_assets(state.assets, None)?;

            let mut style = (*state.ui.inner.style()).clone();
            style.text_styles = [
                (TextStyle::Heading, (FontId { size: 32.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Body, (FontId { size: 20.0, family: FontFamily::Name("Kenney Pixel".into()) })),
                (TextStyle::Button, (FontId { size: 32.0, family: FontFamily::Name("Kenney Pixel".into()) })),
            ]
            .into();
            state.ui.inner.set_style(style);

            return Ok(Some(FrameCommand::ChangeScene { name: "LoadingScene".to_string() }));
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<FullOutput> {
        Ok(state.ui.inner.run(input, |_| {}))
    }
}
