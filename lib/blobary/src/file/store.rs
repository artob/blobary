// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobStore, BlobStoreExt, Result};
use std::io::Read;

#[derive(Default)]
pub struct FileBlobStore {}

impl BlobStore for FileBlobStore {
    fn size(&self) -> BlobID {
        todo!("size not implemented yet") // TODO
    }

    fn hash_to_id(&self, _blob_hash: BlobHash) -> Option<BlobID> {
        todo!("hash_to_id not implemented yet") // TODO
    }

    fn id_to_hash(&self, _blob_id: BlobID) -> Option<BlobHash> {
        todo!("id_to_hash not implemented yet") // TODO
    }

    fn get_by_id(&self, _blob_id: BlobID) -> Option<Blob> {
        todo!("get_by_id not implemented yet") // TODO
    }

    fn get_by_hash(&self, _blob_hash: BlobHash) -> Option<Blob> {
        todo!("get_by_hash not implemented yet") // TODO
    }

    fn put(&mut self, _blob_data: &mut dyn Read) -> Result<Blob> {
        todo!("put not implemented yet") // TODO
    }

    fn remove(&mut self, _blob_hash: BlobHash) -> Result<bool> {
        todo!("remove not implemented yet") // TODO
    }
}

impl BlobStoreExt for FileBlobStore {}
