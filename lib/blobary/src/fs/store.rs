// This is free and unencumbered software released into the public domain.

use crate::{Blob, BlobHash, BlobID, BlobStore, File, PersistentBlobRecord, Result, RECORD_SIZE};
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

pub struct PersistentBlobStore {
    pub(crate) dir: Dir,
    pub(crate) index_file: RefCell<Box<dyn File>>, // .blobary.index
    pub(crate) lookup_id: HashMap<BlobHash, BlobID>,
}

impl PersistentBlobStore {
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
                Err(err) => return Err(err.into()),
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

    pub(crate) fn read_record(&self, blob_id: BlobID) -> Option<PersistentBlobRecord> {
        let index_file = self.index_file.borrow();

        let record_id: usize = blob_id - 1;
        let mut buffer = [0u8; RECORD_SIZE];
        index_file
            .read_exact_at(&mut buffer, (record_id * RECORD_SIZE) as u64)
            .err()?;

        let record = PersistentBlobRecord::read_from(&buffer)?;
        Some(record)
    }
}

impl BlobStore for PersistentBlobStore {
    fn size(&self) -> BlobID {
        // TODO: remove the dependence on #![feature(seek_stream_len)]
        self.index_file.borrow_mut().stream_len().unwrap() as BlobID / RECORD_SIZE as BlobID
    }

    fn hash_to_id(&self, blob_hash: BlobHash) -> Option<BlobID> {
        self.lookup_id.get(&blob_hash).copied()
    }

    fn id_to_hash(&self, blob_id: BlobID) -> Option<BlobHash> {
        let blob_record = self.read_record(blob_id)?;
        Some(blob_record.0.into())
    }

    fn get_by_hash(&self, blob_hash: BlobHash) -> Option<Rc<RefCell<dyn Blob>>> {
        if !self.lookup_id.contains_key(&blob_hash) {
            return None;
        }
        let blob_filename = blob_hash.to_hex();
        let blob_file = self.dir.open(blob_filename.as_str()).ok()?;
        Some(Rc::new(RefCell::new(blob_file)))
    }

    fn get_by_id(&self, blob_id: BlobID) -> Option<Rc<RefCell<dyn Blob>>> {
        self.get_by_hash(self.id_to_hash(blob_id)?)
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<BlobID> {
        // Buffer the blob data in a temporary file:
        let mut temp_file = TempFile::new(&self.dir)?;
        let blob_size = std::io::copy(blob_data, &mut temp_file)?;

        // Compute the BLAKE3 hash for the blob data:
        let mut blob_hasher = blake3::Hasher::new();
        temp_file.rewind()?;
        std::io::copy(&mut temp_file, &mut blob_hasher)?;

        // Check if the blob is already in the store:
        let blob_hash = blob_hasher.finalize();
        if let Some(blob_id) = self.lookup_id.get(&blob_hash) {
            return Ok(*blob_id);
        }

        // Rename the temporary file to its final name:
        temp_file.as_file().set_permissions(Permissions::from_std(
            std::fs::Permissions::from_mode(0o444),
        ))?;
        temp_file.replace(blob_hash.to_hex().as_str())?;

        let blob_id: BlobID = self.lookup_id.len() + 1;
        let blob_record = PersistentBlobRecord(blob_hash.into(), blob_size.into());

        // Write the blob metadata to the index:
        let mut index_file = self.index_file.borrow_mut();
        index_file.seek(std::io::SeekFrom::End(0))?;
        index_file.write_all(blob_record.as_bytes())?;
        index_file.sync_all()?;
        self.lookup_id.insert(blob_hash, blob_id);

        Ok(blob_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let temp_dir = cap_tempfile::tempdir(ambient_authority()).unwrap();
        let mut store = PersistentBlobStore::open_tempdir(&temp_dir).unwrap();
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

        // eprintln!("{}", std::env::temp_dir().to_str().unwrap());
        // std::process::exit(0); // leave `temp_dir`` around for inspection
    }
}
