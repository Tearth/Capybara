use crate::assets::RawSound;
use crate::utils::storage::StorageItem;
use anyhow::Result;
use kira::sound::static_sound::StaticSoundData;
use kira::sound::static_sound::StaticSoundSettings;
use kira::track::TrackId;
use kira::OutputDestination;
use std::io::Cursor;

pub struct Sound {
    pub id: usize,
    pub name: Option<String>,
    pub inner: StaticSoundData,
}

impl Sound {
    pub fn new(raw: &RawSound, track: Option<TrackId>) -> Result<Self> {
        let cursor = Cursor::new(raw.data.clone());
        let inner = StaticSoundData::from_cursor(cursor, StaticSoundSettings::default())?;

        if let Some(track_id) = track {
            inner.settings.output_destination(OutputDestination::Track(track_id));
        }

        Ok(Self { id: 0, name: None, inner })
    }
}

impl StorageItem for Sound {
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
}
