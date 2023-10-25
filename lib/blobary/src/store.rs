// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobIterator, BlobStoreError};
use std::{io::Read, path::Path};

pub type Result<T> = std::result::Result<T, BlobStoreError>;

pub trait BlobStore {
    /// Returns the number of blobs in this store.
    fn count(&self) -> Result<usize>;

    /// Converts a BLAKE3 hash to a store ID.
    fn hash_to_id(&self, blob_hash: BlobHash) -> Result<Option<BlobID>>;

    /// Converts a store ID to a BLAKE3 hash.
    fn id_to_hash(&self, blob_id: BlobID) -> Result<Option<BlobHash>>;

    /// Determines if the store contains a blob with the given BLAKE3 hash.
    fn contains_hash(&self, blob_hash: BlobHash) -> Result<bool>;

    /// Fetches a blob by its store ID.
    fn get_by_id(&self, blob_id: BlobID) -> Result<Option<Blob>>;

    /// Fetches a blob by its BLAKE3 hash.
    fn get_by_hash(&self, blob_hash: BlobHash) -> Result<Option<Blob>>;

    /// Stores a blob and returns its metadata.
    fn put(&mut self, blob_data: &mut dyn Read) -> Result<(bool, Blob)>;

    /// Removes a blob by its BLAKE3 hash.
    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool>;
}

pub trait BlobStoreExt: BlobStore {
    /// Determines if the store contains no blobs.
    fn is_empty(&self) -> Result<bool> {
        Ok(self.count()? == 0)
    }

    /// Stores a blob and returns its store ID.
    fn put_string(&mut self, data: impl AsRef<str>) -> Result<(bool, Blob)> {
        self.put(&mut data.as_ref().as_bytes())
    }

    /// Stores a blob and returns its store ID.
    fn put_file(&mut self, path: impl AsRef<Path>) -> Result<(bool, Blob)> {
        self.put(&mut std::fs::File::open(path)?)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BlobStoreOptions {
    pub writable: bool,
}

impl Default for BlobStoreOptions {
    fn default() -> Self {
        Self { writable: true }
    }
}

impl BlobStoreExt for dyn BlobStore {}

impl<'a> IntoIterator for &'a mut dyn BlobStore {
    type Item = Blob;
    type IntoIter = BlobIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BlobIterator::new(self)
    }
}
