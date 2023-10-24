// This is free and unencumbered software released into the public domain.

use crate::{
    encode_into_path, Blob, BlobHash, BlobHasher, BlobID, BlobStore, BlobStoreExt, File,
    PersistentBlobRecord, Result, RECORD_SIZE,
};
use cap_std::{
    ambient_authority,
    fs::{Dir, Permissions},
};
use cap_tempfile::{TempDir, TempFile};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{ErrorKind::UnexpectedEof, Read, Seek, Write},
    os::unix::prelude::PermissionsExt,
    path::Path,
    rc::Rc,
};
use zerocopy::{AsBytes, FromBytes};

const INDEX_FILE_NAME: &str = ".blobary.index";

pub struct DirectoryBlobStore {
    pub(crate) dir: Dir,
    pub(crate) index_file: RefCell<Box<dyn File>>, // .blobary.index
    pub(crate) lookup_id: HashMap<BlobHash, BlobID>,
}

impl DirectoryBlobStore {
    pub fn open_cwd() -> Result<Self> {
        Self::open_path(".")
    }

    pub fn open_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_dir(Dir::open_ambient_dir(path, ambient_authority())?)
    }

    pub fn open_tempdir(dir: &TempDir) -> Result<Self> {
        Self::open_dir(dir.open_dir(".")?)
    }

    pub fn open_dir(dir: Dir) -> Result<Self> {
        let mut options = cap_std::fs::File::options();
        let options = options.create(true).read(true).append(true);

        // Open the index file:
        let mut index_file = dir.open_with(INDEX_FILE_NAME, options)?;
        index_file.set_permissions(Permissions::from_std(std::fs::Permissions::from_mode(
            0o644,
        )))?;

        // Load the hash-to-ID lookup map from the index file:
        let mut lookup_id = HashMap::new();
        let mut buffer = [0u8; RECORD_SIZE];
        index_file.rewind()?;
        loop {
            match index_file.read_exact(&mut buffer) {
                Ok(_) => (),
                Err(err) if err.kind() == UnexpectedEof => break,
                Err(err) => return Err(Box::new(err)),
            }
            let record = PersistentBlobRecord::read_from(&buffer).unwrap();
            lookup_id.insert(record.0.into(), lookup_id.len() + 1);
        }

        Ok(Self {
            dir,
            index_file: RefCell::new(Box::new(index_file)),
            lookup_id,
        })
    }

    pub(crate) fn read_record(&self, blob_id: BlobID) -> Result<Option<PersistentBlobRecord>> {
        let index_file = self.index_file.borrow();
        let record_id: usize = blob_id - 1;
        let mut buffer = [0u8; RECORD_SIZE];
        match index_file.read_exact_at(&mut buffer, (record_id * RECORD_SIZE) as u64) {
            Ok(()) => (),
            Err(err) if err.kind() == UnexpectedEof => return Ok(None),
            Err(err) => return Err(err.into()),
        }
        Ok(PersistentBlobRecord::read_from(&buffer))
    }
}

impl BlobStore for DirectoryBlobStore {
    fn count(&self) -> Result<BlobID> {
        // TODO: remove the dependence on #![feature(seek_stream_len)]
        Ok(self.index_file.borrow_mut().stream_len().unwrap() as BlobID / RECORD_SIZE as BlobID)
    }

    fn hash_to_id(&self, blob_hash: BlobHash) -> Result<Option<BlobID>> {
        Ok(self.lookup_id.get(&blob_hash).copied())
    }

    fn id_to_hash(&self, blob_id: BlobID) -> Result<Option<BlobHash>> {
        Ok(self
            .read_record(blob_id)?
            .map(|blob_record| blob_record.0.into()))
    }

    fn get_by_id(&self, blob_id: BlobID) -> Result<Option<Blob>> {
        match self.id_to_hash(blob_id)? {
            None => Ok(None),
            Some(blob_hash) => self.get_by_hash(blob_hash),
        }
    }

    fn get_by_hash(&self, blob_hash: BlobHash) -> Result<Option<Blob>> {
        match self.lookup_id.get(&blob_hash) {
            None => Ok(None),
            Some(blob_id) => {
                let blob_path = encode_into_path(blob_hash);
                let mut blob_file = self.dir.open(blob_path)?;
                Ok(Some(Blob {
                    id: *blob_id,
                    hash: blob_hash,
                    size: blob_file.stream_len()?,
                    data: Some(Rc::new(RefCell::new(blob_file))),
                }))
            }
        }
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<Blob> {
        // Buffer the blob data in a temporary file:
        let mut temp_file = TempFile::new(&self.dir)?;
        let blob_size = std::io::copy(blob_data, &mut temp_file)?;

        // Compute the BLAKE3 hash for the blob data:
        let mut blob_hasher = BlobHasher::new();
        temp_file.rewind()?;
        std::io::copy(&mut temp_file, &mut blob_hasher)?;

        // Check if the blob is already in the store:
        let blob_hash = blob_hasher.finalize();
        if let Some(blob_id) = self.lookup_id.get(&blob_hash) {
            return Ok(Blob {
                id: *blob_id,
                hash: blob_hash,
                size: blob_size,
                data: None,
            });
        }

        // Rename the temporary file to its final name:
        let blob_path = encode_into_path(blob_hash);
        temp_file.as_file().set_permissions(Permissions::from_std(
            std::fs::Permissions::from_mode(0o444),
        ))?;
        temp_file.replace(blob_path)?;

        let blob_id: BlobID = self.lookup_id.len() + 1;
        let blob_record = PersistentBlobRecord(blob_hash.into(), blob_size.into());

        // Write the blob metadata to the index:
        let mut index_file = self.index_file.borrow_mut();
        index_file.seek(std::io::SeekFrom::End(0))?;
        index_file.write_all(blob_record.as_bytes())?;
        index_file.sync_all()?;
        self.lookup_id.insert(blob_hash, blob_id);

        Ok(Blob {
            id: blob_id,
            hash: blob_hash,
            size: blob_size,
            data: None,
        })
    }

    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool> {
        match self.lookup_id.remove(&blob_hash) {
            None => Ok(false), // not found
            Some(_blob_id) => {
                let blob_path = encode_into_path(blob_hash);
                match self.dir.remove_file(blob_path) {
                    Ok(_) => Ok(true),
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
                    Err(err) => Err(Box::new(err)),
                }
            }
        }
    }
}

impl BlobStoreExt for DirectoryBlobStore {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let temp_dir = cap_tempfile::tempdir(ambient_authority()).unwrap();
        let mut store = DirectoryBlobStore::open_tempdir(&temp_dir).unwrap();
        assert_eq!(store.count().unwrap(), 0);

        let foo = store.put_string("Foo").unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(foo.id, 1);

        let foo2 = store.put_string("Foo").unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(foo2.id, 1);

        let bar = store.put_string("Bar").unwrap();
        assert_eq!(store.count().unwrap(), 2);
        assert_eq!(bar.id, 2);

        // eprintln!("{}", std::env::temp_dir().to_str().unwrap());
        // std::process::exit(0); // leave `temp_dir`` around for inspection
    }
}
