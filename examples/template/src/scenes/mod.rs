use capybara::assets::loader::AssetsLoader;
use capybara::kira::track::TrackHandle;
use capybara::utils::settings::SettingsStorage;

pub mod boot;
pub mod game;
pub mod loading;
pub mod menu;

pub const SETTINGS_MASTER_VOLUME: &str = "MASTER_VOLUME";
pub const SETTINGS_MUSIC_VOLUME: &str = "MUSIC_VOLUME";
pub const SETTINGS_EFFECTS_VOLUME: &str = "EFFECTS_VOLUME";

pub struct GlobalData {
    pub assets: AssetsLoader,
    pub settings: SettingsStorage,

    pub music_track: Option<TrackHandle>,
    pub effects_track: Option<TrackHandle>,
}

impl Default for GlobalData {
    fn default() -> Self {
        Self { assets: Default::default(), settings: SettingsStorage::new("./settings.cfg"), music_track: None, effects_track: None }
    }
}
