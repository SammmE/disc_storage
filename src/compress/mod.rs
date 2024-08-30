use std::{fs::File, path::PathBuf, sync::mpsc};

use serde::{Deserialize, Serialize};
use tar::Builder;

pub mod lzma_compress;
pub mod zstd_compress;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CompressType {
    LZMA,
    Zstd,
}

pub trait Compress {
    fn compress(
        &self,
        output: PathBuf,
        files: Vec<PathBuf>,
        level: i32,
        updater: mpsc::Sender<(f32, f32)>,
    ) -> Result<(), std::io::Error>;
    fn decompress(&self, input: PathBuf, output: PathBuf) -> Result<(), std::io::Error>;
}

fn make_tar(output: PathBuf, files: Vec<PathBuf>) -> Result<(), std::io::Error> {
    let mut archive = Builder::new(File::create(output)?);
    for file in files {
        archive.append_path(file)?;
    }
    Ok(())
}
