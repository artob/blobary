// This is free and unencumbered software released into the public domain.

use crate::sysexits::Sysexits;
use blobary::DirectoryBlobStore;

pub fn open_store() -> Result<DirectoryBlobStore, Sysexits> {
    match DirectoryBlobStore::open_cwd() {
        Ok(store) => Ok(store),
        Err(err) => {
            eprintln!("{}: {}", "blobary", err);
            Err(Sysexits::EX_IOERR)
        }
    }
}
