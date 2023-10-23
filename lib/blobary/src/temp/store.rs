// This is free and unencumbered software released into the public domain.

use crate::{hash, Blob, BlobHash, BlobID, BlobStore, BlobStoreExt, Result};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Cursor, Read},
    rc::Rc,
};

#[derive(Default)]
pub struct EphemeralBlobStore {
    index: HashMap<BlobHash, BlobID>,
    store: Vec<Blob>,
}

impl EphemeralBlobStore {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }
}

impl BlobStore for EphemeralBlobStore {
    fn size(&self) -> BlobID {
        self.store.len() as BlobID
    }

    fn hash_to_id(&self, blob_hash: BlobHash) -> Option<BlobID> {
        self.index.get(&blob_hash).copied()
    }

    fn id_to_hash(&self, blob_id: BlobID) -> Option<BlobHash> {
        match blob_id {
            0 => None,
            _ => self.store.get(blob_id - 1).map(|blob| blob.hash),
        }
    }

    fn get_by_id(&self, blob_id: BlobID) -> Option<Blob> {
        match blob_id {
            0 => None,
            _ => self.store.get(blob_id - 1).cloned(),
        }
    }

    fn get_by_hash(&self, blob_hash: BlobHash) -> Option<Blob> {
        match self.index.get(&blob_hash) {
            None => None,
            Some(blob_id) => self.get_by_id(*blob_id),
        }
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<Blob> {
        let mut buffer = Vec::new();
        blob_data.read_to_end(&mut buffer)?;
        let blob_size = buffer.len() as u64;

        let blob_hash = hash(&buffer);
        if let Some(blob_id) = self.index.get(&blob_hash) {
            return match self.get_by_id(*blob_id) {
                None => unreachable!("blob_id {} not found", blob_id),
                Some(blob) => Ok(blob),
            };
        }

        let blob_id: BlobID = self.store.len() + 1;
        let blob_data = Rc::new(RefCell::new(Cursor::new(buffer)));
        let blob = Blob {
            id: blob_id,
            hash: blob_hash,
            size: blob_size,
            data: Some(blob_data),
        };

        self.store.push(blob.clone());
        self.index.insert(blob_hash, blob_id);

        Ok(blob)
    }

    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool> {
        match self.hash_to_id(blob_hash) {
            None => Ok(false), // not found
            Some(blob_id) => {
                self.index.remove(&blob_hash);
                self.store.remove(blob_id);
                Ok(true)
            }
        }
    }
}

impl BlobStoreExt for EphemeralBlobStore {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut store = EphemeralBlobStore::default();
        assert_eq!(store.size(), 0);

        let foo = store.put_string("Foo").unwrap();
        assert_eq!(store.size(), 1);
        assert_eq!(foo.id, 1);

        let foo2 = store.put_string("Foo").unwrap();
        assert_eq!(store.size(), 1);
        assert_eq!(foo2.id, 1);

        let bar = store.put_string("Bar").unwrap();
        assert_eq!(store.size(), 2);
        assert_eq!(bar.id, 2);
    }
}
