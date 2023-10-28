// This is free and unencumbered software released into the public domain.

use crate::{hash::encode_hash, sysexits::Sysexits, Options};
use blobary::{IndexedBlobStore, IndexedBlobStoreIterator};
use std::ops::DerefMut;

pub fn copy_blobs(
    source_store: &mut Box<dyn IndexedBlobStore>,
    target_store: &mut Box<dyn IndexedBlobStore>,
    options: &Options,
) -> Result<usize, Sysexits> {
    let mutate_count: usize = 0;

    for blob in IndexedBlobStoreIterator::new(source_store.deref_mut()) {
        let blob_data = blob.data.unwrap();
        let mut blob_data = blob_data.borrow_mut();

        if target_store.contains_hash(blob.hash)? {
            continue;
        }

        let (created, _) = target_store.put(&mut blob_data.deref_mut())?;
        if created && (options.verbose || options.debug) {
            println!("{}", encode_hash(blob.hash));
        }
    }

    Ok(mutate_count)
}
