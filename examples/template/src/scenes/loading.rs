use super::GlobalData;
use super::*;
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
use capybara::kira::track::TrackBuilder;
use capybara::kira::tween::Tween;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::InputEvent;

#[derive(Default)]
pub struct LoadingScene {}

impl Scene<GlobalData> for LoadingScene {
    fn activation(&mut self, state: ApplicationState<GlobalData>) -> Result<()> {
        let music_track = state.audio.inner.add_sub_track(TrackBuilder::new())?;
        let effects_track = state.audio.inner.add_sub_track(TrackBuilder::new())?;

        state.global.settings.set(SETTINGS_MASTER_VOLUME, 1.0, false);
        state.global.settings.set(SETTINGS_MUSIC_VOLUME, 1.0, false);
        state.global.settings.set(SETTINGS_EFFECTS_VOLUME, 1.0, false);

        let master_volume = state.global.settings.get::<f64>(SETTINGS_MASTER_VOLUME)?;
        let music_volume = state.global.settings.get::<f64>(SETTINGS_MUSIC_VOLUME)?;
        let effects_volume = state.global.settings.get::<f64>(SETTINGS_EFFECTS_VOLUME)?;

        music_track.set_volume(music_volume * master_volume, Tween::default())?;
        effects_track.set_volume(effects_volume * master_volume, Tween::default())?;

        state.global.music_track = Some(music_track);
        state.global.effects_track = Some(effects_track);

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
        if state.global.assets.load("./data/main.zip") == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(&state.global.assets, None);
            state.ui.instantiate_assets(&state.global.assets, None);

            let music_track_id = state.global.music_track.as_ref().unwrap().id();
            let effects_track_id = state.global.effects_track.as_ref().unwrap().id();

            state.audio.instantiate_assets(&state.global.assets, Some("/music/"), Some(music_track_id));
            state.audio.instantiate_assets(&state.global.assets, Some("/sounds/"), Some(effects_track_id));

            return Ok(Some(FrameCommand::ChangeScene { name: "MenuScene".to_string() }));
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().unwrap().run(input, |context| {
            CentralPanel::default().show(context, |ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                    ui.label(RichText::new("Loading...".to_string()).heading().color(Color32::from_rgb(255, 255, 255)));
                });
            });
        });

        Ok((output, None))
    }
}
