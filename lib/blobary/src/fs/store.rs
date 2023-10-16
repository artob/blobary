// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobStore, Result};
use std::{io::Read, rc::Rc};

#[derive(Default)]
pub struct PersistentBlobStore {}

impl BlobStore for PersistentBlobStore {
    fn size(&self) -> BlobID {
        todo!() // TODO
    }

    fn get_by_hash(&self, _blob_hash: BlobHash) -> Option<Rc<dyn Blob>> {
        todo!() // TODO
    }

    fn get_by_id(&self, _blob_id: BlobID) -> Option<Rc<dyn Blob>> {
        todo!() // TODO
    }

    fn put(&mut self, _blob_data: &mut dyn Read) -> Result<BlobID> {
        todo!() // TODO
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let _store = PersistentBlobStore::default();
        //assert_eq!(store.size(), 0);
    }
}
