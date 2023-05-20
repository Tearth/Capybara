use super::*;
use crate::filesystem::FileLoadingStatus;
use crate::filesystem::FileSystem;
use anyhow::bail;
use anyhow::Result;
use png::Decoder;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::borrow::Cow;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::str;
use zip::ZipArchive;

pub struct AssetsLoader {
    pub input: String,
    pub status: AssetsLoadingStatus,
    pub filesystem: FileSystem,

    pub raw_textures: Vec<RawTexture>,
    pub raw_fonts: Vec<RawFont>,
    pub raw_atlases: Vec<RawAtlas>,
    pub raw_sounds: Vec<RawSound>,
}

impl AssetsLoader {
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            status: AssetsLoadingStatus::Idle,
            filesystem: Default::default(),
            raw_textures: Default::default(),
            raw_fonts: Default::default(),
            raw_atlases: Default::default(),
            raw_sounds: Default::default(),
        }
    }

    pub fn load(&mut self, input: &str) -> Result<AssetsLoadingStatus> {
        if self.status == AssetsLoadingStatus::Idle && self.input != input {
            self.status = AssetsLoadingStatus::Initializing;
        }

        match self.status {
            AssetsLoadingStatus::Initializing => {
                self.filesystem.load(input)?;

                self.input = input.to_string();
                self.status = AssetsLoadingStatus::Loading;
            }
            AssetsLoadingStatus::Loading => {
                if self.filesystem.load(input)? == FileLoadingStatus::Finished {
                    let buffer = self.filesystem.buffer.clone();
                    let buffer = buffer.borrow();

                    let slice = buffer.as_slice();
                    let cursor = Cursor::new(slice);
                    let mut archive = ZipArchive::new(cursor)?;

                    for i in 0..archive.len() {
                        let mut entry = archive.by_index(i)?;
                        if entry.is_file() {
                            let path = Path::new(entry.name());
                            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                            let extension = path.extension().unwrap().to_str().unwrap().to_string();

                            let mut data = Vec::new();
                            entry.read_to_end(&mut data)?;

                            match extension.as_str() {
                                "png" => self.load_png(&name, &data)?,
                                "ttf" => self.load_ttf(&name, &data)?,
                                "xml" => self.load_xml(&name, &data)?,
                                "wav" => self.load_wav(&name, &data)?,
                                "ogg" => self.load_ogg(&name, &data)?,
                                _ => {}
                            };
                        }
                    }

                    self.status = AssetsLoadingStatus::Finished;
                }
            }
            AssetsLoadingStatus::Finished => {
                self.status = AssetsLoadingStatus::Idle;
            }
            _ => {}
        }

        Ok(self.status)
    }

    fn load_png(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let cursor = Cursor::new(data);
        let mut decoder = Decoder::new(cursor);
        decoder.set_transformations(png::Transformations::normalize_to_color8());

        let mut reader = decoder.read_info()?;
        let mut data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut data)?;
        let size = Vec2::new(info.width as f32, info.height as f32);

        self.raw_textures.push(RawTexture::new(name, size, &data));

        Ok(())
    }

    fn load_ttf(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.raw_fonts.push(RawFont::new(name, data));
        Ok(())
    }

    fn load_xml(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let xml = str::from_utf8(data)?;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        reader.expand_empty_elements(true);

        let mut image = String::new();
        let mut entities = Vec::new();

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

                        entities.push(RawAtlasEntity::new(&name, position, size));
                    }
                    _ => {}
                },
                Err(error) => bail!("Error at position {}: {:?}", reader.buffer_position(), error),
                Ok(Event::Eof) => break,
                _ => (),
            }
        }

        self.raw_atlases.push(RawAtlas::new(name, &image, entities));

        Ok(())
    }

    fn load_wav(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.raw_sounds.push(RawSound::new(name, data));
        Ok(())
    }

    fn load_ogg(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.raw_sounds.push(RawSound::new(name, data));
        Ok(())
    }
}

impl Default for AssetsLoader {
    fn default() -> Self {
        Self::new()
    }
}
