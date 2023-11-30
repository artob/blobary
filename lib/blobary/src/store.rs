// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobStoreError, Filter, IndexedBlobStoreIterator};
use std::{io::Read, path::Path};

pub type Result<T> = std::result::Result<T, BlobStoreError>;

pub trait BlobStore {
    /// Returns the number of blobs in this store.
    fn count(&self) -> Result<usize>;

    /// Determines if the store contains a blob with the given BLAKE3 hash.
    fn contains_hash(&self, blob_hash: BlobHash) -> Result<bool>;

    /// Fetches a blob by its BLAKE3 hash.
    fn get_by_hash(&self, blob_hash: BlobHash) -> Result<Option<Blob>>;

    /// Stores a blob and returns its metadata.
    fn put(&mut self, blob_data: &mut dyn Read) -> Result<(bool, Blob)>;

    /// Removes a blob by its BLAKE3 hash.
    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool>;
}

pub trait IndexedBlobStore: BlobStore {
    /// Converts a BLAKE3 hash to a store ID.
    fn hash_to_id(&self, blob_hash: BlobHash) -> Result<Option<BlobID>>;

    /// Converts a store ID to a BLAKE3 hash.
    fn id_to_hash(&self, blob_id: BlobID) -> Result<Option<BlobHash>>;

    /// Fetches a blob by its store ID.
    fn get_by_id(&self, blob_id: BlobID) -> Result<Option<Blob>>;
}

pub trait BlobStoreExt: BlobStore {
    /// Determines if the store contains no blobs.
    fn is_empty(&self) -> Result<bool> {
        Ok(self.count()? == 0)
    }

    /// Stores a blob and returns its store ID.
    fn put_bytes(&mut self, data: impl AsRef<[u8]>) -> Result<(bool, Blob)> {
        self.put(&mut data.as_ref())
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

#[derive(Clone, Debug)]
pub struct BlobStoreOptions {
    pub writable: bool,
    pub filters: Vec<Box<dyn Filter>>,
}

impl Default for BlobStoreOptions {
    fn default() -> Self {
        Self {
            writable: true,
            filters: vec![],
        }
    }
}

impl BlobStoreOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn filters(mut self, filters: Vec<Box<dyn Filter>>) -> Self {
        self.filters = filters;
        self
    }

    pub fn filter(mut self, filter: Box<dyn Filter>) -> Self {
        self.filters.push(filter);
        self
    }
}

impl BlobStoreExt for dyn BlobStore {}

impl BlobStoreExt for dyn IndexedBlobStore {}

impl<'a> IntoIterator for &'a mut dyn IndexedBlobStore {
    type Item = Blob;
    type IntoIter = IndexedBlobStoreIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedBlobStoreIterator::new(self)
    }
}
