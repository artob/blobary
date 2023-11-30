// This is free and unencumbered software released into the public domain.

use crate::{
    encode_into_path, Blob, BlobHash, BlobHasher, BlobID, BlobStore, BlobStoreError, BlobStoreExt,
    BlobStoreOptions, File, IndexedBlobStore, PersistentBlobRecord, Result, RECORD_SIZE,
};
use cap_std::{
    ambient_authority,
    fs::{Dir, Permissions},
};
use cap_tempfile::{TempDir, TempFile};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::create_dir_all,
    io::{ErrorKind::UnexpectedEof, Read, Seek, SeekFrom, Write},
    os::unix::prelude::PermissionsExt,
    path::Path,
    rc::Rc,
};
use zerocopy::{AsBytes, FromBytes};

const STORE_DIR_NAME: &str = ".blobary";
const INDEX_FILE_NAME: &str = ".index";

pub struct DirectoryBlobStore {
    pub(crate) config: BlobStoreOptions,
    dir: Dir,                           // .blobary
    index_file: RefCell<Box<dyn File>>, // .blobary/index
    lookup_id: HashMap<BlobHash, BlobID>,
}

impl DirectoryBlobStore {
    pub fn open_in_cwd(config: BlobStoreOptions) -> Result<Self> {
        Self::open_path(STORE_DIR_NAME, config)
    }

    pub fn open_path(path: impl AsRef<Path>, config: BlobStoreOptions) -> Result<Self> {
        if config.writable {
            create_dir_all(path.as_ref())?;
        }
        Self::open_dir(Dir::open_ambient_dir(path, ambient_authority())?, config)
    }

    pub fn open_tempdir(temp_dir: &TempDir, config: BlobStoreOptions) -> Result<Self> {
        Self::open_dir(temp_dir.open_dir(".")?, config)
    }

    pub fn open_dir(dir: Dir, config: BlobStoreOptions) -> Result<Self> {
        let mut index_options = cap_std::fs::File::options();
        let index_options = index_options
            .create(config.writable)
            .read(true)
            .append(true);

        // Open the index file:
        let mut index_file = dir.open_with(INDEX_FILE_NAME, index_options)?;
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
            config,
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
        let mut stream = self.index_file.borrow_mut();
        Ok(stream_len(stream.as_mut()).unwrap() as BlobID / RECORD_SIZE as BlobID)
    }

    fn contains_hash(&self, blob_hash: BlobHash) -> Result<bool> {
        Ok(self.lookup_id.contains_key(&blob_hash))
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
                    size: stream_len(&mut blob_file)?,
                    data: Some(Rc::new(RefCell::new(blob_file))),
                }))
            }
        }
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<(bool, Blob)> {
        if !self.config.writable {
            return Err(crate::BlobStoreError::NotWritable.into());
        }

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
            return Ok((
                false,
                Blob {
                    id: *blob_id,
                    hash: blob_hash,
                    size: blob_size,
                    data: None,
                },
            ));
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

        Ok((
            true,
            Blob {
                id: blob_id,
                hash: blob_hash,
                size: blob_size,
                data: None,
            },
        ))
    }

    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool> {
        if !self.config.writable {
            return Err(crate::BlobStoreError::NotWritable.into());
        }

        match self.lookup_id.remove(&blob_hash) {
            None => Ok(false), // not found
            Some(_blob_id) => {
                let blob_path = encode_into_path(blob_hash);
                match self.dir.remove_file(blob_path) {
                    Ok(_) => Ok(true),
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }
}

impl IndexedBlobStore for DirectoryBlobStore {
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
            Some(blob_hash) => {
                let blob_path = encode_into_path(blob_hash);
                let mut blob_file = match self.dir.open(blob_path) {
                    Ok(blob_file) => blob_file,
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        // The index entry remains, but the actual blob file has been removed:
                        return Err(BlobStoreError::Removed);
                    }
                    Err(err) => return Err(err.into()),
                };
                Ok(Some(Blob {
                    id: blob_id,
                    hash: blob_hash,
                    size: stream_len(&mut blob_file)?,
                    data: Some(Rc::new(RefCell::new(blob_file))),
                }))
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
        let mut store =
            DirectoryBlobStore::open_tempdir(&temp_dir, BlobStoreOptions::default()).unwrap();
        assert_eq!(store.count().unwrap(), 0);

        let (_, foo) = store.put_string("Foo").unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(foo.id, 1);

        let (_, foo2) = store.put_string("Foo").unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(foo2.id, 1);

        let (_, bar) = store.put_string("Bar").unwrap();
        assert_eq!(store.count().unwrap(), 2);
        assert_eq!(bar.id, 2);

        // eprintln!("{}", std::env::temp_dir().to_str().unwrap());
        // std::process::exit(0); // leave `temp_dir`` around for inspection
    }
}

#[allow(clippy::seek_from_current)]
fn stream_len<T: Seek + ?Sized>(stream: &mut T) -> Result<u64> {
    let old_pos = stream.seek(SeekFrom::Current(0))?;
    let len = stream.seek(SeekFrom::End(0))?;
    if old_pos != len {
        stream.seek(SeekFrom::Start(old_pos))?;
    }
    Ok(len)
}
