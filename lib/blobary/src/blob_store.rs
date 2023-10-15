// This is free and unencumbered software released into the public domain.

use crate::blob::{Blob, BlobHash, BlobID};
use std::{io::Read, rc::Rc};

pub use std::io::Result;

pub trait BlobStore {
    /// Returns the number of blobs in this store.
    fn size(&self) -> usize;

    /// Fetch a blob by its BLAKE3 hash.
    fn get_by_hash(&self, blob_hash: BlobHash) -> Option<Rc<dyn Blob>>;

    /// Fetch a blob by its store ID.
    fn get_by_id(&self, blob_id: BlobID) -> Option<Rc<dyn Blob>>;

    /// Store a blob and return its store ID.
    fn put(&mut self, blob_data: &mut dyn Read) -> Result<BlobID>;
}
