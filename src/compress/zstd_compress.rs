use std::{fs::File, path::PathBuf};

use zstd::stream::{copy_decode, copy_encode};

use super::{make_tar, Compress};

pub struct ZstdCompress;

impl Compress for ZstdCompress {
    fn compress(
        &self,
        output: PathBuf,
        files: Vec<PathBuf>,
        level: i32,
    ) -> Result<(), std::io::Error> {
        make_tar(
            output.clone().parent().unwrap().join("archive.tar"),
            files.clone(),
        )?;
        let mut archive = File::open(output.parent().unwrap().join("archive.tar"))?;
        let mut compressed = File::create(output)?;
        copy_encode(&mut archive, &mut compressed, level)?;
        Ok(())
    }

    fn decompress(&self, input: PathBuf, output: PathBuf) -> Result<(), std::io::Error> {
        let mut compressed = File::open(input)?;
        let mut archive = File::create(output.clone().parent().unwrap().join("archive.tar"))?;
        copy_decode(&mut compressed, &mut archive)?;
        Ok(())
    }
}
