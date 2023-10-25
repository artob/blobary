// This is proprietary and confidential source code not for distribution.

use crate::{Blob, BlobStore};
use std::iter::Iterator;

pub struct BlobIterator<'a> {
    pub(crate) store: &'a mut dyn BlobStore,
    pub(crate) index: usize,
    pub(crate) count: usize,
}

impl<'a> BlobIterator<'a> {
    pub fn new(store: &'a mut dyn BlobStore) -> Self {
        let count = store.count().unwrap();
        Self {
            store,
            index: 0,
            count,
        }
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = Blob;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index <= self.count {
            self.store.get_by_id(self.index).unwrap() // FIXME: handle deleted blobs
        } else {
            None
        }
    }
}
