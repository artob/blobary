// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobStore, BlobStoreExt, Result};
use std::io::Read;

#[derive(Default)]
pub struct SQLiteBlobStore {}

impl BlobStore for SQLiteBlobStore {
    fn size(&self) -> BlobID {
        todo!() // TODO
    }

    fn hash_to_id(&self, _blob_hash: BlobHash) -> Option<BlobID> {
        todo!() // TODO
    }

    fn id_to_hash(&self, _blob_id: BlobID) -> Option<BlobHash> {
        todo!() // TODO
    }

    fn get_by_hash(&self, _blob_hash: BlobHash) -> Option<Blob> {
        todo!() // TODO
    }

    fn get_by_id(&self, _blob_id: BlobID) -> Option<Blob> {
        todo!() // TODO
    }

    fn put(&mut self, _blob_data: &mut dyn Read) -> Result<Blob> {
        todo!() // TODO
    }

    fn remove(&mut self, _blob_hash: BlobHash) -> Result<bool> {
        todo!() // TODO
    }
}

impl BlobStoreExt for SQLiteBlobStore {}
