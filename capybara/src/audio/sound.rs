use crate::assets::RawSound;
use anyhow::Result;
use kira::sound::static_sound::StaticSoundData;
use kira::sound::static_sound::StaticSoundSettings;
use kira::track::TrackId;
use kira::OutputDestination;
use std::io::Cursor;

pub struct Sound {
    pub inner: StaticSoundData,
}

impl Sound {
    pub fn new(raw: &RawSound, track: Option<TrackId>) -> Result<Self> {
        let cursor = Cursor::new(raw.data.clone());
        let inner = StaticSoundData::from_cursor(cursor, StaticSoundSettings::default())?;

        if let Some(track_id) = track {
            inner.settings.output_destination(OutputDestination::Track(track_id));
        }

        Ok(Self { inner })
    }
}
