// This is free and unencumbered software released into the public domain.

use crate::{
    hash, Blob, BlobHash, BlobID, BlobStore, BlobStoreExt, BlobStoreOptions, IndexedBlobStore,
    Result,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Cursor, Read},
    rc::Rc,
};

#[derive(Default)]
pub struct EphemeralBlobStore {
    pub(crate) config: BlobStoreOptions,
    index: HashMap<BlobHash, BlobID>,
    store: Vec<Blob>,
}

impl EphemeralBlobStore {
    #[allow(unused)]
    pub fn new(config: BlobStoreOptions) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}

impl BlobStore for EphemeralBlobStore {
    fn count(&self) -> Result<BlobID> {
        Ok(self.store.len() as BlobID)
    }

    fn contains_hash(&self, blob_hash: BlobHash) -> Result<bool> {
        Ok(self.index.contains_key(&blob_hash))
    }

    fn get_by_hash(&self, blob_hash: BlobHash) -> Result<Option<Blob>> {
        match self.index.get(&blob_hash) {
            None => Ok(None),
            Some(blob_id) => self.get_by_id(*blob_id),
        }
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<(bool, Blob)> {
        if !self.config.writable {
            return Err(crate::BlobStoreError::NotWritable.into());
        }

        let mut buffer = Vec::new();
        blob_data.read_to_end(&mut buffer)?;
        let blob_size = buffer.len() as u64;

        let blob_hash = hash(&buffer);
        if let Some(blob_id) = self.index.get(&blob_hash) {
            return match self.get_by_id(*blob_id)? {
                None => unreachable!("blob_id {} not found", blob_id),
                Some(blob) => Ok((false, blob)),
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

        Ok((true, blob))
    }

    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool> {
        if !self.config.writable {
            return Err(crate::BlobStoreError::NotWritable.into());
        }

        match self.hash_to_id(blob_hash)? {
            None => Ok(false), // not found
            Some(blob_id) => {
                self.index.remove(&blob_hash);
                self.store.remove(blob_id);
                Ok(true)
            }
        }
    }
}

impl IndexedBlobStore for EphemeralBlobStore {
    fn hash_to_id(&self, blob_hash: BlobHash) -> Result<Option<BlobID>> {
        Ok(self.index.get(&blob_hash).copied())
    }

    fn id_to_hash(&self, blob_id: BlobID) -> Result<Option<BlobHash>> {
        Ok(match blob_id {
            0 => None,
            _ => self.store.get(blob_id - 1).map(|blob| blob.hash),
        })
    }

    fn get_by_id(&self, blob_id: BlobID) -> Result<Option<Blob>> {
        Ok(match blob_id {
            0 => None,
            _ => self.store.get(blob_id - 1).cloned(),
        })
    }
}

impl BlobStoreExt for EphemeralBlobStore {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut store = EphemeralBlobStore::default();
        assert_eq!(store.count().unwrap(), 0);

        let (_, foo) = store.put_string("Foo").unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(foo.id, 1);

        let (_, foo2) = store.put_string("Foo").unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(foo2.id, 1);

        let (_, bar) = store.put_string("Bar").unwrap();
        assert_eq!(store.count().unwrap(), 2);
        assert_eq!(bar.id, 2);
    }
}
