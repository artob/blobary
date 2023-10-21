// This is free and unencumbered software released into the public domain.

use crate::BlobHasher;
use std::{
    cell::RefCell,
    io::{Cursor, Read, Result, Seek, Write},
    rc::Rc,
};

pub const DEFAULT_MIME_TYPE: &str = "application/octet-stream";

/// A blob's locally-unique sequence ID in a [`BlobStore`].
#[allow(unused)]
pub type BlobID = usize;

/// The blob's globally-unique cryptographic BLAKE3 hash.
#[allow(unused)]
pub type BlobHash = blake3::Hash;

/// TODO
#[derive(Clone)]
pub struct Blob {
    pub id: BlobID,
    pub hash: BlobHash,
    pub size: u64,
    pub data: Option<Rc<RefCell<dyn BlobData>>>,
}

/// A blob is a unique byte sequence of data.
pub trait BlobData: Seek + Read {
    /// Returns the blob's byte size.
    fn size(&mut self) -> Result<u64> {
        self.stream_len()
    }

    /// Guesses the MIME content type of the blob.
    ///
    /// Preserves the stream position prior to calling this method.
    fn mime_type(&mut self) -> Result<Option<&'static str>> {
        let mut buffer = vec![0u8; 0];
        let pos = self.stream_position()?;
        if pos != 0 {
            self.seek(std::io::SeekFrom::Start(0))?;
        }
        self.take(1024).read_to_end(&mut buffer)?;
        self.seek(std::io::SeekFrom::Start(pos))?;
        Ok(infer::get(&buffer).map(|t| t.mime_type()))
    }

    /// Returns the blob's hash.
    ///
    /// Preserves the stream position prior to calling this method.
    fn hash(&mut self) -> Result<BlobHash> {
        let mut hasher = BlobHasher::new();
        let pos = self.stream_position()?;
        if pos != 0 {
            self.seek(std::io::SeekFrom::Start(0))?;
        }
        std::io::copy(self, &mut hasher)?;
        self.seek(std::io::SeekFrom::Start(pos))?;
        let hash = hasher.finalize();
        Ok(hash)
    }
}

/// A mutable blob is a unique byte sequence of data.
pub trait BlobDataMut: BlobData + Write {}

impl BlobData for Cursor<&[u8]> {}

impl BlobData for Cursor<Vec<u8>> {}

impl BlobData for std::fs::File {}

impl BlobData for cap_std::fs::File {}

impl BlobDataMut for Cursor<Vec<u8>> {}

impl BlobDataMut for std::fs::File {}

impl BlobDataMut for cap_std::fs::File {}
