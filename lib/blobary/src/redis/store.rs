// This is free and unencumbered software released into the public domain.

use crate::{
    hash, Blob, BlobHash, BlobID, BlobStore, BlobStoreError, BlobStoreExt, BlobStoreOptions, Result,
};
use redis::Commands;
use std::{cell::RefCell, io::Read, str::FromStr};

pub struct RedisBlobStore {
    connection: RefCell<redis::Connection>,
    count_key: String,
    index_key: String,
    store_key: String,
}

impl RedisBlobStore {
    #[allow(unused)]
    pub fn open(url: impl AsRef<str>, _options: BlobStoreOptions) -> Result<Self> {
        let client = redis::Client::open(url.as_ref())?;
        let connection = client.get_connection()?;
        Ok(Self {
            connection: RefCell::new(connection),
            count_key: "blobs:id".to_string(),
            index_key: "blobs:index".to_string(),
            store_key: "blobs:store".to_string(),
        })
    }
}

impl BlobStore for RedisBlobStore {
    fn count(&self) -> Result<BlobID> {
        let mut conn = self.connection.borrow_mut();
        Ok(conn.zcard(&self.index_key).unwrap_or(0))
    }

    fn hash_to_id(&self, blob_hash: BlobHash) -> Result<Option<BlobID>> {
        let mut conn = self.connection.borrow_mut();
        let blob_hash_str = blob_hash.to_hex();
        match conn.zscore::<&str, &str, Option<BlobID>>(&self.index_key, blob_hash_str.as_str()) {
            Ok(Some(rank)) => Ok(Some(rank)),
            Ok(None) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn id_to_hash(&self, blob_id: BlobID) -> Result<Option<BlobHash>> {
        let mut conn = self.connection.borrow_mut();
        match conn.zrangebyscore::<&str, BlobID, BlobID, Vec<String>>(
            &self.index_key,
            blob_id,
            blob_id,
        ) {
            Ok(mut results) if results.len() == 1 => {
                let result = results.pop().unwrap();
                let blob_hash = BlobHash::from_str(result.as_str()).expect("parse blob hash");
                Ok(Some(blob_hash))
            }
            Ok(_) => unreachable!("ZRANGEBYSCORE should only return 1 result"),
            Err(err) => Err(err.into()),
        }
    }

    fn get_by_id(&self, _blob_id: BlobID) -> Result<Option<Blob>> {
        Err(BlobStoreError::Unimplemented("get_by_id".to_string())) // TODO
    }

    fn get_by_hash(&self, _blob_hash: BlobHash) -> Result<Option<Blob>> {
        Err(BlobStoreError::Unimplemented("get_by_hash".to_string())) // TODO
    }

    fn put(&mut self, blob_data: &mut dyn Read) -> Result<Blob> {
        let mut conn = self.connection.borrow_mut();
        let blob_id: BlobID = match conn.incr(&self.count_key, 1) {
            Ok(value) => value,
            Err(err) => return Err(err.into()),
        };

        let mut buffer = Vec::new();
        blob_data.read_to_end(&mut buffer)?;

        let blob = Blob {
            id: blob_id,
            hash: hash(&buffer),
            size: buffer.len() as u64,
            data: None,
        };
        let blob_hash_str = blob.hash.to_hex();

        let mut cmd = redis::cmd("ZADD");
        let cmd = cmd
            .arg(&self.index_key)
            .arg("NX")
            .arg(blob.id)
            .arg(blob_hash_str.as_str());

        match cmd.query(&mut conn) {
            Ok(0) => Ok(blob),
            Ok(1) => match conn.hset_nx(&self.store_key, blob.id, buffer) {
                Ok(false) => Ok(blob), // already existed
                Ok(true) => Ok(blob),  // just created
                Err(err) => Err(err.into()),
            },
            Ok(_) => unreachable!("ZADD should only return 0 or 1"),
            Err(err) => Err(err.into()),
        }
    }

    fn remove(&mut self, blob_hash: BlobHash) -> Result<bool> {
        match self.hash_to_id(blob_hash)? {
            None => Ok(false), // not found
            Some(blob_id) => {
                let mut conn = self.connection.borrow_mut();
                let blob_hash_str = blob_hash.to_hex();
                match conn.zrem(&self.index_key, blob_hash_str.as_str()) {
                    Ok(0) => Ok(false), // unreachable except in a race
                    Ok(1) => match conn.hdel(&self.store_key, blob_id) {
                        Ok(0) => Ok(false), // unreachable except in a race
                        Ok(1) => Ok(true),
                        Ok(_) => unreachable!("HDEL should only return 0 or 1"),
                        Err(err) => Err(err.into()),
                    },
                    Ok(_) => unreachable!("ZREM should only return 0 or 1"),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }
}

impl BlobStoreExt for RedisBlobStore {}
