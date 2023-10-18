// This is free and unencumbered software released into the public domain.

use crate::{hash, Blob, BlobHash, BlobID, BlobStore, BlobStoreExt, Result};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Cursor, Read},
    rc::Rc,
};

pub(crate) struct EphemeralBlobRecord(BlobHash, Rc<RefCell<dyn Blob>>);

#[derive(Default)]
pub struct EphemeralBlobStore {
    index: HashMap<BlobHash, BlobID>,
    store: Vec<EphemeralBlobRecord>,
}

impl BlobStore for EphemeralBlobStore {
    fn size(&self) -> BlobID {
        self.store.len() as BlobID
    }

    fn hash_to_id(&self, blob_hash: BlobHash) -> Option<BlobID> {
        self.index.get(&blob_hash).copied()
    }

    fn id_to_hash(&self, blob_id: BlobID) -> Option<BlobHash> {
        self.store.get(blob_id).map(|blob_record| blob_record.0)
    }

    fn get_by_hash(&self, blob_hash: BlobHash) -> Option<Rc<RefCell<dyn Blob>>> {
        match self.index.get(&blob_hash) {
            None => None,
            Some(blob_id) => self.get_by_id(*blob_id),
        }
    }

    fn get_by_id(&self, blob_id: BlobID) -> Option<Rc<RefCell<dyn Blob>>> {
        match blob_id {
            0 => None,
            _ => self
                .store
                .get(blob_id - 1)
                .map(|blob_record| Rc::clone(&blob_record.1)),
        }
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<BlobID> {
        let mut buffer = Vec::new();
        blob_data.read_to_end(&mut buffer)?;

        let blob_hash = hash(&buffer);
        if let Some(blob_id) = self.index.get(&blob_hash) {
            return Ok(*blob_id);
        }

        let blob_record =
            EphemeralBlobRecord(blob_hash, Rc::new(RefCell::new(Cursor::new(buffer))));

        let blob_id: BlobID = self.store.len() + 1;
        self.store.push(blob_record);
        self.index.insert(blob_hash, blob_id);
        Ok(blob_id)
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

        let foo_id = store.put_string("Foo").unwrap();
        assert_eq!(store.size(), 1);
        assert_eq!(foo_id, 1);

        let foo2_id = store.put_string("Foo").unwrap();
        assert_eq!(store.size(), 1);
        assert_eq!(foo2_id, 1);

        let bar_id = store.put_string("Bar").unwrap();
        assert_eq!(store.size(), 2);
        assert_eq!(bar_id, 2);
    }
}
