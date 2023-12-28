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
        let sound_track = state.audio.inner.add_sub_track(TrackBuilder::new())?;

        state.global.settings.set(SETTINGS_MUSIC_LEVEL, 1.0, false);
        state.global.settings.set(SETTINGS_SOUND_LEVEL, 1.0, false);

        music_track.set_volume(state.global.settings.get::<f64>(SETTINGS_MUSIC_LEVEL)?, Tween::default())?;
        sound_track.set_volume(state.global.settings.get::<f64>(SETTINGS_SOUND_LEVEL)?, Tween::default())?;

        state.global.music_track = Some(music_track);
        state.global.sound_track = Some(sound_track);

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
            let sound_track_id = state.global.sound_track.as_ref().unwrap().id();

            state.audio.instantiate_assets(&state.global.assets, Some("/music/"), Some(music_track_id));
            state.audio.instantiate_assets(&state.global.assets, Some("/sounds/"), Some(sound_track_id));

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
