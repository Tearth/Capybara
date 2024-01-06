use super::ldtk::LdtkWorld;
use super::*;
use crate::filesystem::FileLoadingStatus;
use crate::filesystem::FileSystem;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use log::error;
use log::info;
use png::Decoder;
use png::Transformations;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::str;
use tinyjson::JsonValue;
use zip::ZipArchive;

pub struct AssetsLoader {
    pub input: String,
    pub status: AssetsLoadingStatus,
    pub filesystem: FileSystem,
    pub current_index: usize,
    pub progress: f32,

    pub raw_textures: Vec<RawTexture>,
    pub raw_fonts: Vec<RawFont>,
    pub raw_atlases: Vec<RawAtlas>,
    pub raw_sounds: Vec<RawSound>,
    pub worlds: Vec<LdtkWorld>,
}

impl AssetsLoader {
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            status: AssetsLoadingStatus::Idle,
            filesystem: Default::default(),
            current_index: 0,
            progress: 0.0,

            raw_textures: Default::default(),
            raw_fonts: Default::default(),
            raw_atlases: Default::default(),
            raw_sounds: Default::default(),
            worlds: Default::default(),
        }
    }

    pub fn load(&mut self, path: &str) -> AssetsLoadingStatus {
        if (self.status == AssetsLoadingStatus::Finished || self.status == AssetsLoadingStatus::Error) && self.input != path {
            self.status = AssetsLoadingStatus::Idle;
            self.current_index = 0;
            self.progress = 0.0;
        }

        match self.status {
            AssetsLoadingStatus::Idle => {
                info!("Loading assets from {}", path);

                self.filesystem.read(path);
                self.input = path.to_string();
                self.status = AssetsLoadingStatus::Loading;
            }
            AssetsLoadingStatus::Loading => {
                match self.filesystem.read(path) {
                    FileLoadingStatus::Finished => {
                        self.status = AssetsLoadingStatus::Parsing;
                    }
                    FileLoadingStatus::Error => {
                        self.status = AssetsLoadingStatus::Error;
                        error!("Failed to load assets file");
                    }
                    _ => {}
                };

                self.progress = *self.filesystem.progress.borrow() / 2.0;
            }
            AssetsLoadingStatus::Parsing => {
                let buffer = self.filesystem.buffer.clone();
                let buffer = buffer.borrow();

                let slice = buffer.as_slice();
                let cursor = Cursor::new(slice);
                let mut archive = match ZipArchive::new(cursor) {
                    Ok(archive) => archive,
                    Err(err) => {
                        self.status = AssetsLoadingStatus::Error;
                        error!("Failed to create archive reader ({})", err);

                        return self.status;
                    }
                };

                if self.current_index == archive.len() {
                    self.status = AssetsLoadingStatus::Finished;
                    return self.status;
                }

                let mut data = Vec::new();
                let entries_count = archive.len();

                let mut entry = match archive.by_index(self.current_index) {
                    Ok(entry) => entry,
                    Err(err) => {
                        self.current_index += 1;
                        error!("Failed to read archive file ({})", err);

                        return self.status;
                    }
                };

                self.current_index += 1;
                self.progress = 0.5 + (self.current_index as f32 / entries_count as f32 / 2.0);

                if entry.is_file() {
                    let path = Path::new(entry.name());
                    let path_str = match path.to_str() {
                        Some(name) => name.to_string(),
                        None => {
                            error!("Failed to get string path {:?}", path);
                            return self.status;
                        }
                    };
                    let name = match path.file_stem().and_then(|p| p.to_str()) {
                        Some(name) => name.to_string(),
                        None => {
                            error!("Failed to get name from path {:?}", path);
                            return self.status;
                        }
                    };
                    let extension = match path.extension().and_then(|p| p.to_str()) {
                        Some(extension) => extension.to_string(),
                        None => {
                            error!("Failed to get extension from path {:?}", path);
                            return self.status;
                        }
                    };
                    let asset_path = format!("/{}", path_str);

                    data.clear();

                    if let Err(err) = entry.read_to_end(&mut data) {
                        error!("Failed to read data from archive file ({})", err);
                        return self.status;
                    }

                    let result = match extension.as_str() {
                        "png" => Some(self.load_png(&name, &asset_path, &data)),
                        "ttf" => Some(self.load_ttf(&name, &asset_path, &data)),
                        "xml" => Some(self.load_xml(&name, &asset_path, &data)),
                        "ldtk" => Some(self.load_ldtk(&name, &asset_path, &data)),
                        "wav" => Some(self.load_wav(&name, &asset_path, &data)),
                        "ogg" => Some(self.load_ogg(&name, &asset_path, &data)),
                        _ => None,
                    };

                    match result {
                        Some(Ok(())) => info!("Asset {} loaded ({} bytes)", entry.name(), entry.size()),
                        Some(Err(err)) => error!("Failed to load asset {} ({})", entry.name(), err),
                        None => info!("Asset {} skipped (extension not supported)", entry.name()),
                    };
                }
            }
            AssetsLoadingStatus::Finished => {
                self.status = AssetsLoadingStatus::Idle;
            }
            _ => {}
        }

        self.status
    }

    fn load_png(&mut self, name: &str, path: &str, data: &[u8]) -> Result<()> {
        let cursor = Cursor::new(data);
        let mut decoder = Decoder::new(cursor);
        decoder.set_transformations(Transformations::normalize_to_color8());

        let mut reader = decoder.read_info()?;
        let mut data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut data)?;
        let size = Vec2::new(info.width as f32, info.height as f32);

        for i in 0..data.len() / 4 {
            let r = data[i * 4 + 0] as f32 / 255.0;
            let g = data[i * 4 + 1] as f32 / 255.0;
            let b = data[i * 4 + 2] as f32 / 255.0;
            let a = data[i * 4 + 3] as f32 / 255.0;

            data[i * 4 + 0] = (r * a * 255.0) as u8;
            data[i * 4 + 1] = (g * a * 255.0) as u8;
            data[i * 4 + 2] = (b * a * 255.0) as u8;
        }

        self.raw_textures.push(RawTexture::new(name, path, size, &data));

        Ok(())
    }

    fn load_ttf(&mut self, name: &str, path: &str, data: &[u8]) -> Result<()> {
        self.raw_fonts.push(RawFont::new(name, path, data));
        Ok(())
    }

    fn load_xml(&mut self, name: &str, path: &str, data: &[u8]) -> Result<()> {
        let xml = str::from_utf8(data)?;
        let mut image = String::new();
        let mut entities = Vec::new();
        let mut reader = Reader::from_str(xml);

        reader.trim_text(true);
        reader.expand_empty_elements(true);

        loop {
            match reader.read_event() {
                Ok(Event::Start(element)) => match element.name().as_ref() {
                    b"TextureAtlas" => {
                        let mut image_path = Cow::Borrowed("");

                        for attr_result in element.attributes() {
                            let a = attr_result?;
                            if let b"imagePath" = a.key.as_ref() {
                                image_path = a.decode_and_unescape_value(&reader)?
                            }
                        }

                        image = image_path.to_string();
                    }
                    b"SubTexture" => {
                        let mut name = Cow::Borrowed("");
                        let mut x = Cow::Borrowed("");
                        let mut y = Cow::Borrowed("");
                        let mut width = Cow::Borrowed("");
                        let mut height = Cow::Borrowed("");

                        for attr_result in element.attributes() {
                            let a = attr_result?;
                            match a.key.as_ref() {
                                b"name" => name = a.decode_and_unescape_value(&reader)?,
                                b"x" => x = a.decode_and_unescape_value(&reader)?,
                                b"y" => y = a.decode_and_unescape_value(&reader)?,
                                b"width" => width = a.decode_and_unescape_value(&reader)?,
                                b"height" => height = a.decode_and_unescape_value(&reader)?,
                                _ => (),
                            }
                        }

                        let position = Vec2::new(x.parse()?, y.parse()?);
                        let size = Vec2::new(width.parse()?, height.parse()?);

                        entities.push(RawAtlasEntity::new(&name, path, position, size));
                    }
                    _ => {}
                },
                Err(error) => bail!("Error at position {}: {:?}", reader.buffer_position(), error),
                Ok(Event::Eof) => break,
                _ => (),
            }
        }

        self.raw_atlases.push(RawAtlas::new(name, path, &image, entities));

        Ok(())
    }

    fn load_ldtk(&mut self, name: &str, path: &str, data: &[u8]) -> Result<()> {
        let json = str::from_utf8(data)?.parse::<JsonValue>()?;
        let data = json.get::<HashMap<_, _>>().ok_or_else(|| anyhow!("Failed to read JSON data"))?;
        self.worlds.push(ldtk::load_world(name, path, data)?);

        Ok(())
    }

    fn load_wav(&mut self, name: &str, path: &str, data: &[u8]) -> Result<()> {
        self.raw_sounds.push(RawSound::new(name, path, data));
        Ok(())
    }

    fn load_ogg(&mut self, name: &str, path: &str, data: &[u8]) -> Result<()> {
        self.raw_sounds.push(RawSound::new(name, path, data));
        Ok(())
    }
}

impl Default for AssetsLoader {
    fn default() -> Self {
        Self::new()
    }
}
