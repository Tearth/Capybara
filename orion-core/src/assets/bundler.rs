use anyhow::Result;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

pub fn pack(input: &str, output: &str) -> Result<()> {
    let output_path = PathBuf::from(output);
    let output_dir = output_path.parent().unwrap();
    fs::create_dir_all(output_dir)?;

    let output_file = File::create(output).unwrap();
    let mut zip = ZipWriter::new(output_file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated).unix_permissions(0o755);

    let walkdir = WalkDir::new(input);
    let mut buffer = Vec::new();

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let mut zip_path = path.strip_prefix(input).unwrap().to_str().unwrap().to_string();
        zip_path = zip_path.replace('\\', "/");

        if !zip_path.is_empty() {
            if path.is_file() {
                let mut file = File::open(path)?;

                zip.start_file(zip_path, options)?;
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;

                buffer.clear();
            } else if path.is_dir() {
                zip.add_directory(zip_path, options)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}
