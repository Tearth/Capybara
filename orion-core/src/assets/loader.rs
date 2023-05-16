use super::*;
use crate::filesystem::FileLoadingStatus;
use crate::filesystem::FileSystem;
use anyhow::Result;
use png::Decoder;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub struct AssetsLoader {
    pub input: String,
    pub status: AssetsLoadingStatus,
    pub filesystem: FileSystem,

    pub raw_textures: Vec<RawTexture>,
    pub raw_fonts: Vec<RawFont>,
}

impl AssetsLoader {
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            status: AssetsLoadingStatus::Idle,
            filesystem: Default::default(),
            raw_textures: Default::default(),
            raw_fonts: Default::default(),
        }
    }

    pub fn load(&mut self, input: &str) -> Result<AssetsLoadingStatus> {
        if self.status == AssetsLoadingStatus::Finished && self.input != input {
            self.status = AssetsLoadingStatus::Idle;
        }

        match self.status {
            AssetsLoadingStatus::Idle => {
                self.filesystem.load(input)?;

                self.input = input.to_string();
                self.status = AssetsLoadingStatus::Loading;
            }
            AssetsLoadingStatus::Loading => {
                if self.filesystem.load(input)? == FileLoadingStatus::Finished {
                    let buffer = self.filesystem.buffer.borrow();
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

                            if extension == "png" {
                                let cursor = Cursor::new(data);
                                let mut decoder = Decoder::new(cursor);
                                decoder.set_transformations(png::Transformations::normalize_to_color8());

                                let mut reader = decoder.read_info()?;
                                let mut data = vec![0; reader.output_buffer_size()];
                                let info = reader.next_frame(&mut data)?;
                                let size = Vec2::new(info.width as f32, info.height as f32);

                                self.raw_textures.push(RawTexture::new(name, size, &data));
                            } else if extension == "ttf" {
                                self.raw_fonts.push(RawFont::new(name, data));
                            }
                        }
                    }

                    self.status = AssetsLoadingStatus::Finished;
                }
            }
            _ => {}
        }

        Ok(self.status)
    }
}

impl Default for AssetsLoader {
    fn default() -> Self {
        Self::new()
    }
}
