// This is free and unencumbered software released into the public domain.

use crate::{error::BlobHashError, BlobHasher};
use std::{
    cell::RefCell,
    io::{Cursor, Read, Result, Seek, SeekFrom, Write},
    rc::Rc,
};

pub const BLOB_HASH_LEN: usize = 32;
pub const DEFAULT_MIME_TYPE: &str = "application/octet-stream";

/// A blob's locally-unique sequence ID in a [`BlobStore`].
#[allow(unused)]
pub type BlobID = usize;

/// The blob's globally-unique cryptographic BLAKE3 hash.
#[allow(unused)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct BlobHash(pub blake3::Hash);

pub type BlobHashResult<T> = std::result::Result<T, BlobHashError>;

impl BlobHash {
    pub fn zero() -> Self {
        Self::from_bytes([0u8; BLOB_HASH_LEN])
    }

    pub fn from_vec(bytes: &Vec<u8>) -> BlobHashResult<Self> {
        match bytes.len() {
            BLOB_HASH_LEN => Ok(Self::from_bytes(bytes.as_slice().try_into().unwrap())),
            n => Err(BlobHashError::InvalidLength(n)),
        }
    }

    pub fn from_slice(bytes: &[u8]) -> BlobHashResult<Self> {
        match bytes.len() {
            BLOB_HASH_LEN => Ok(Self::from_bytes(bytes.try_into().unwrap())),
            n => Err(BlobHashError::InvalidLength(n)),
        }
    }

    pub fn from_bytes(bytes: [u8; BLOB_HASH_LEN]) -> Self {
        Self(blake3::Hash::from_bytes(bytes))
    }

    pub fn from_hex(input: impl AsRef<[u8]>) -> BlobHashResult<Self> {
        let input = input.as_ref();
        match blake3::Hash::from_hex(input) {
            Ok(hash) => Ok(Self(hash)),
            Err(_) => Err(BlobHashError::InvalidInput(
                String::from_utf8_lossy(input).to_string(),
            )),
        }
    }

    #[cfg(feature = "base58")]
    pub fn from_base58(input: impl AsRef<str>) -> BlobHashResult<Self> {
        let input = input.as_ref();
        match bs58::decode(input).into_vec() {
            Ok(bytes) => Self::from_vec(&bytes),
            Err(_) => Err(BlobHashError::InvalidInput(input.to_string())),
        }
    }

    /// Returns the blob's hash as a hex string.
    pub fn to_hex(&self) -> arrayvec::ArrayString<64> {
        self.0.to_hex()
    }
}

impl std::str::FromStr for BlobHash {
    type Err = BlobHashError;

    fn from_str(input: &str) -> BlobHashResult<Self> {
        Self::from_hex(input).or_else(|_| {
            #[cfg(feature = "base58")]
            return Self::from_base58(input);
            #[cfg(not(feature = "base58"))]
            return Err(BlobHashError::InvalidInput(input.to_string()));
        })
    }
}

impl From<[u8; BLOB_HASH_LEN]> for BlobHash {
    fn from(bytes: [u8; BLOB_HASH_LEN]) -> Self {
        Self(blake3::Hash::from(bytes))
    }
}

impl From<BlobHash> for [u8; BLOB_HASH_LEN] {
    fn from(blob_hash: BlobHash) -> Self {
        blob_hash.0.into()
    }
}

impl std::fmt::Debug for BlobHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl std::fmt::Display for BlobHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

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
        let old_pos = self.seek(SeekFrom::Current(0))?;
        let len = self.seek(SeekFrom::End(0))?;
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }
        Ok(len)
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
        Ok(hasher.finalize())
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
