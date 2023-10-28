// This is proprietary and confidential source code not for distribution.

use crate::{Blob, BlobStore, BlobStoreError};
use std::iter::Iterator;

pub struct IndexedBlobStoreIterator<'a> {
    pub(crate) store: &'a mut dyn BlobStore,
    pub(crate) index: usize,
    pub(crate) count: usize,
}

impl<'a> IndexedBlobStoreIterator<'a> {
    pub fn new(store: &'a mut dyn BlobStore) -> Self {
        let count = store.count().unwrap();
        Self {
            store,
            index: 0,
            count,
        }
    }
}

impl<'a> Iterator for IndexedBlobStoreIterator<'a> {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.index += 1;
            if self.index > self.count {
                return None;
            }
            match self.store.get_by_id(self.index) {
                Ok(None) => unreachable!(),
                Ok(Some(blob)) => return Some(blob),
                Err(BlobStoreError::Removed) => continue,
                Err(err) => panic!("Failed to read blob #{}: {}", self.index, err),
            }
        }
    }
}
