use capybara::assets::loader::AssetsLoader;
use capybara::kira::track::TrackHandle;
use capybara::utils::settings::SettingsStorage;

pub mod boot;
pub mod game;
pub mod loading;
pub mod menu;

pub const SETTINGS_MUSIC_LEVEL: &str = "MUSIC_LEVEL";
pub const SETTINGS_SOUND_LEVEL: &str = "SOUND_LEVEL";

pub struct GlobalData {
    pub assets: AssetsLoader,
    pub settings: SettingsStorage,

    pub music_track: Option<TrackHandle>,
    pub sound_track: Option<TrackHandle>,
}

impl Default for GlobalData {
    fn default() -> Self {
        Self { assets: Default::default(), settings: SettingsStorage::new("./settings.cfg"), music_track: None, sound_track: None }
    }
}
