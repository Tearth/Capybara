use super::sound::Sound;
use crate::assets::loader::AssetsLoader;
use crate::error_continue;
use crate::utils::storage::Storage;
use anyhow::Result;
use kira::manager::backend::cpal::CpalBackend;
use kira::manager::AudioManager;
use kira::manager::AudioManagerSettings;
use kira::track::TrackId;
use log::error;

pub struct AudioContext {
    pub inner: AudioManager<CpalBackend>,
    pub sounds: Storage<Sound>,
}

impl AudioContext {
    pub fn new() -> Result<Self> {
        Ok(Self { inner: AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?, sounds: Default::default() })
    }

    pub fn instantiate_assets(&mut self, assets: &AssetsLoader, prefix: Option<&str>, track: Option<TrackId>) {
        for raw in &assets.raw_sounds {
            if let Some(prefix) = &prefix {
                if !raw.path.starts_with(prefix) {
                    continue;
                }
            }

            let sound = match Sound::new(raw, track) {
                Ok(sound) => sound,
                Err(err) => error_continue!("Failed to create sound {} ({})", raw.name, err),
            };

            if self.sounds.store_with_name(&raw.name, sound).is_err() {
                error!("Failed to instantiate sound {}", raw.name);
            }
        }
    }
}
