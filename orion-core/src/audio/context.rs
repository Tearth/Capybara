use super::sound::Sound;
use crate::assets::loader::AssetsLoader;
use crate::utils::storage::Storage;
use anyhow::Result;
use kira::manager::backend::cpal::CpalBackend;
use kira::manager::AudioManager;
use kira::manager::AudioManagerSettings;

pub struct AudioContext {
    pub inner: AudioManager<CpalBackend>,
    pub sounds: Storage<Sound>,
}

impl AudioContext {
    pub fn new() -> Result<Self> {
        Ok(Self { inner: AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?, sounds: Default::default() })
    }

    pub fn instantiate_assets(&mut self, assets: &AssetsLoader) -> Result<()> {
        for sound in &assets.raw_sounds {
            self.sounds.store_with_name(&sound.name, Sound::new(sound)?)?;
        }

        Ok(())
    }
}
