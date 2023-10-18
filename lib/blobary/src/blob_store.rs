// This is free and unencumbered software released into the public domain.

use crate::blob::{Blob, BlobHash, BlobID};
use std::{cell::RefCell, io::Read, rc::Rc};

pub use std::io::Result;

pub trait BlobStore {
    /// Returns the number of blobs in this store.
    fn size(&self) -> usize;

    /// TODO
    fn hash_to_id(&self, blob_hash: BlobHash) -> Option<BlobID>;

    /// TODO
    fn id_to_hash(&self, blob_id: BlobID) -> Option<BlobHash>;

    /// Fetches a blob by its BLAKE3 hash.
    fn get_by_hash(&self, blob_hash: BlobHash) -> Option<Rc<RefCell<dyn Blob>>>;

    /// Fetches a blob by its store ID.
    fn get_by_id(&self, blob_id: BlobID) -> Option<Rc<RefCell<dyn Blob>>>;

    /// Stores a blob and return its store ID.
    fn put(&mut self, blob_data: &mut dyn Read) -> Result<BlobID>;

    /// Stores a blob and return its store ID.
    fn put_string(&mut self, blob_data: impl AsRef<str>) -> Result<BlobID> {
        self.put(&mut blob_data.as_ref().as_bytes())
    }
}
