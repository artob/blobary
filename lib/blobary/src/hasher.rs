// This is free and unencumbered software released into the public domain.

use crate::BlobHash;
use std::path::Path;

pub fn hash(input: impl AsRef<[u8]>) -> BlobHash {
    BlobHash(blake3::hash(input.as_ref()))
}

#[derive(Default)]
pub struct BlobHasher(pub blake3::Hasher);

impl BlobHasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finalize(self) -> BlobHash {
        BlobHash(self.0.finalize())
    }

    pub fn update_from_path(&mut self, input_path: impl AsRef<Path>) -> std::io::Result<&mut Self> {
        self.0.update_mmap(input_path)?;
        Ok(self)
    }
}

impl std::io::Write for BlobHasher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
