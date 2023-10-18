// This is free and unencumbered software released into the public domain.

use crate::BlobHasher;
use std::io::{Cursor, Read, Seek, Write};

/// A blob's locally-unique sequence ID in a [`BlobStore`].
#[allow(unused)]
pub type BlobID = usize;

/// The blob's globally-unique cryptographic BLAKE3 hash.
#[allow(unused)]
pub type BlobHash = blake3::Hash;

/// A blob is a unique byte sequence of data.
pub trait Blob: Seek + Read {
    /// Returns the blob's byte size.
    fn size(&mut self) -> Result<u64, std::io::Error> {
        self.stream_len()
    }

    /// Returns the blob's hash.
    fn hash(&mut self) -> Result<BlobHash, std::io::Error> {
        let mut hasher = BlobHasher::new();
        std::io::copy(self, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(hash)
    }
}

/// A mutable blob is a unique byte sequence of data.
pub trait BlobMut: Blob + Write {}

impl Blob for Cursor<&[u8]> {}

impl Blob for Cursor<Vec<u8>> {}

impl Blob for std::fs::File {}

impl Blob for cap_std::fs::File {}

impl BlobMut for Cursor<Vec<u8>> {}

impl BlobMut for std::fs::File {}

impl BlobMut for cap_std::fs::File {}
