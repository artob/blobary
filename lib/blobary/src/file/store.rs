// This is free and unencumbered software released into the public domain.

use crate::{
    Blob, BlobHash, BlobID, BlobStore, BlobStoreError, BlobStoreExt, IndexedBlobStore, Result,
};
use std::io::Read;

#[derive(Default)]
pub struct FileBlobStore {}

impl BlobStore for FileBlobStore {
    fn count(&self) -> Result<BlobID> {
        Err(BlobStoreError::Unimplemented("count".to_string())) // TODO
    }

    fn contains_hash(&self, _blob_hash: BlobHash) -> Result<bool> {
        Err(BlobStoreError::Unimplemented("contains_hash".to_string())) // TODO
    }

    fn get_by_hash(&self, _blob_hash: BlobHash) -> Result<Option<Blob>> {
        Err(BlobStoreError::Unimplemented("get_by_hash".to_string())) // TODO
    }

    fn put(&mut self, _blob_data: &mut dyn Read) -> Result<(bool, Blob)> {
        Err(BlobStoreError::Unimplemented("put".to_string())) // TODO
    }

    fn remove(&mut self, _blob_hash: BlobHash) -> Result<bool> {
        Err(BlobStoreError::Unimplemented("remove".to_string())) // TODO
    }
}

impl IndexedBlobStore for FileBlobStore {
    fn hash_to_id(&self, _blob_hash: BlobHash) -> Result<Option<BlobID>> {
        Err(BlobStoreError::Unimplemented("hash_to_id".to_string())) // TODO
    }

    fn id_to_hash(&self, _blob_id: BlobID) -> Result<Option<BlobHash>> {
        Err(BlobStoreError::Unimplemented("id_to_hash".to_string())) // TODO
    }

    fn get_by_id(&self, _blob_id: BlobID) -> Result<Option<Blob>> {
        Err(BlobStoreError::Unimplemented("get_by_id".to_string())) // TODO
    }
}

impl BlobStoreExt for FileBlobStore {}
