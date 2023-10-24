// This is free and unencumbered software released into the public domain.

use crate::{
    hash, Blob, BlobHash, BlobID, BlobStore, BlobStoreError, BlobStoreExt, BlobStoreOptions, Result,
};
use s3::creds::Credentials;
use std::{
    cell::RefCell,
    io::{Cursor, Read},
    rc::Rc,
};

pub struct S3BlobStore {
    bucket: s3::Bucket,
    prefix: String,
}

impl S3BlobStore {
    #[allow(unused)]
    pub fn open(
        bucket: impl AsRef<str>,
        prefix: impl AsRef<str>,
        _options: BlobStoreOptions,
    ) -> Result<Self> {
        let bucket = s3::Bucket::new(
            bucket.as_ref(),
            s3::Region::UsEast1,     // TODO: support other regions
            Credentials::default()?, // TODO: support other credentials
        )?;
        Ok(Self {
            bucket,
            prefix: prefix.as_ref().to_string(),
        })
    }
}

impl BlobStore for S3BlobStore {
    fn count(&self) -> Result<BlobID> {
        Err(BlobStoreError::Unsupported)
    }

    fn hash_to_id(&self, _blob_hash: BlobHash) -> Result<Option<BlobID>> {
        Err(BlobStoreError::Unsupported)
    }

    fn id_to_hash(&self, _blob_id: BlobID) -> Result<Option<BlobHash>> {
        Err(BlobStoreError::Unsupported)
    }

    fn get_by_id(&self, _blob_id: BlobID) -> Result<Option<Blob>> {
        Err(BlobStoreError::Unsupported)
    }

    fn get_by_hash(&self, blob_hash: BlobHash) -> Result<Option<Blob>> {
        let blob_path = format!("{}/{}", self.prefix, blob_hash);

        match self.bucket.get_object(blob_path) {
            Err(err) => Err(err.into()),
            Ok(response) => {
                match response.status_code() {
                    404 => Ok(None), // not found
                    200 => {
                        let blob_data = response.bytes().to_vec();
                        let blob_size = blob_data.len();
                        let blob_data = Cursor::new(blob_data);
                        let blob_data = Rc::new(RefCell::new(blob_data));
                        Ok(Some(Blob {
                            id: 0, // FIXME
                            hash: blob_hash,
                            size: blob_size as _,
                            data: Some(blob_data),
                        }))
                    }
                    _ => Err(BlobStoreError::Unexpected.into()),
                }
            }
        }
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<Blob> {
        let mut buffer = Vec::new();
        blob_data.read_to_end(&mut buffer)?;

        let blob = Blob {
            id: 0, // FIXME
            hash: hash(&buffer),
            size: buffer.len() as u64,
            data: None,
        };
        let blob_path = format!("{}/{}", self.prefix, blob.hash);

        match self
            .bucket
            .put_object(blob_path.as_str(), buffer.as_slice())
        {
            Ok(_response) => Ok(blob),
            Err(err) => Err(err.into()),
        }
    }

    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool> {
        let blob_path = format!("{}/{}", self.prefix, blob_hash);

        match self.bucket.delete_object(blob_path.as_str()) {
            Ok(_response) => Ok(true), // can't determine if it existed or not
            Err(err) => Err(err.into()),
        }
    }
}

impl BlobStoreExt for S3BlobStore {}
