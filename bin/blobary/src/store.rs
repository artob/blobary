// This is free and unencumbered software released into the public domain.

use crate::sysexits::Sysexits;
use blobary::{BlobStore, DirectoryBlobStore};

pub fn open_store() -> Result<Box<dyn BlobStore>, Sysexits> {
    match DirectoryBlobStore::open_cwd() {
        Ok(store) => Ok(Box::new(store)),
        Err(err) => {
            eprintln!("blobary: {}", err);
            Err(Sysexits::EX_IOERR)
        }
    }
}
