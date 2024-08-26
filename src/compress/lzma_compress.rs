use std::{fs::File, path::PathBuf};

use xz2::read::{XzDecoder, XzEncoder};

use super::{make_tar, Compress};

struct LzmaCompress;

impl Compress for LzmaCompress {
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
        let archive = File::open(output.parent().unwrap().join("archive.tar"))?;
        let mut compressed = File::create(output)?;
        let mut encoder = XzEncoder::new(archive, level.try_into().unwrap());
        std::io::copy(&mut encoder, &mut compressed)?;
        Ok(())
    }

    fn decompress(&self, input: PathBuf, output: PathBuf) -> Result<(), std::io::Error> {
        let compressed = File::open(input)?;
        let mut archive = File::create(output.clone().parent().unwrap().join("archive.tar"))?;
        let mut decoder = XzDecoder::new(compressed);
        std::io::copy(&mut decoder, &mut archive)?;
        Ok(())
    }
}
