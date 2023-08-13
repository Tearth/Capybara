use super::sound::Sound;
use crate::assets::loader::AssetsLoader;
use crate::utils::storage::Storage;
use anyhow::Result;
use kira::manager::backend::cpal::CpalBackend;
use kira::manager::AudioManager;
use kira::manager::AudioManagerSettings;
use kira::track::TrackId;

pub struct AudioContext {
    pub inner: AudioManager<CpalBackend>,
    pub sounds: Storage<Sound>,
}

impl AudioContext {
    pub fn new() -> Result<Self> {
        Ok(Self { inner: AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?, sounds: Default::default() })
    }

    pub fn instantiate_assets(&mut self, assets: &AssetsLoader, prefix: Option<&str>, track: Option<TrackId>) -> Result<()> {
        for sound in &assets.raw_sounds {
            if let Some(prefix) = &prefix {
                if !sound.path.starts_with(prefix) {
                    continue;
                }
            }

            self.sounds.store_with_name(&sound.name, Sound::new(sound, track)?)?;
        }

        Ok(())
    }
}
