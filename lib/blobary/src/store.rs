// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobIterator};
use std::{cell::RefCell, io::Read, path::Path, rc::Rc};

pub use std::io::Result;

pub trait BlobStore {
    /// Returns the number of blobs in this store.
    fn size(&self) -> usize;

    /// Converts a BLAKE3 hash to a store ID.
    fn hash_to_id(&self, blob_hash: BlobHash) -> Option<BlobID>;

    /// Converts a store ID to a BLAKE3 hash.
    fn id_to_hash(&self, blob_id: BlobID) -> Option<BlobHash>;

    /// Fetches a blob by its BLAKE3 hash.
    fn get_by_hash(&self, blob_hash: BlobHash) -> Option<Rc<RefCell<dyn Blob>>>;

    /// Fetches a blob by its store ID.
    fn get_by_id(&self, blob_id: BlobID) -> Option<Rc<RefCell<dyn Blob>>>;

    /// Stores a blob and returns its store ID.
    fn put(&mut self, blob_data: &mut dyn Read) -> Result<BlobID>;
}

pub trait BlobStoreExt: BlobStore {
    /// Stores a blob and returns its store ID.
    fn put_string(&mut self, data: impl AsRef<str>) -> Result<BlobID> {
        self.put(&mut data.as_ref().as_bytes())
    }

    /// Stores a blob and returns its store ID.
    fn put_file(&mut self, path: impl AsRef<Path>) -> Result<BlobID> {
        self.put(&mut std::fs::File::open(path)?)
    }
}

impl<'a> IntoIterator for &'a mut dyn BlobStore {
    type Item = Rc<RefCell<(dyn Blob + 'static)>>;
    type IntoIter = BlobIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BlobIterator::new(self)
    }
}