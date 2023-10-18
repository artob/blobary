// This is proprietary and confidential source code not for distribution.

use crate::{Blob, BlobStore};
use std::{cell::RefCell, iter::Iterator, rc::Rc};

pub struct BlobIterator<'a> {
    pub(crate) store: &'a mut dyn BlobStore,
    pub(crate) index: usize,
    pub(crate) count: usize,
}

impl<'a> BlobIterator<'a> {
    pub fn new(store: &'a mut dyn BlobStore) -> Self {
        let count = store.size();
        Self {
            store,
            index: 0,
            count,
        }
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = Rc<RefCell<(dyn Blob + 'static)>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index <= self.count {
            self.store.get_by_id(self.index)
        } else {
            None
        }
    }
}
