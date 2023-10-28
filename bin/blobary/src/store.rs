// This is free and unencumbered software released into the public domain.

use crate::sysexits::Sysexits;
use blobary::{BlobStore, BlobStoreOptions, DirectoryBlobStore, EphemeralBlobStore};
use std::env::VarError;
use url::Url;

pub fn open_store(writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    match std::env::var("BLOBARY_URL") {
        Err(VarError::NotPresent) => open_store_in_cwd(writable),
        Err(VarError::NotUnicode(_)) => {
            eprintln!("blobary: BLOBARY_URL contains invalid UTF-8");
            Err(Sysexits::EX_DATAERR)
        }
        Ok(url) => {
            if url.is_empty() {
                open_store_in_cwd(writable)
            } else {
                open_store_from_url(url, writable)
            }
        }
    }
}

fn open_store_in_cwd(writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    match DirectoryBlobStore::open_in_cwd(BlobStoreOptions { writable }) {
        Ok(store) => Ok(Box::new(store)),
        Err(err) => {
            eprintln!("blobary: {}", err);
            Err(Sysexits::EX_IOERR)
        }
    }
}

pub fn open_store_from_url(
    url: impl AsRef<str>,
    writable: bool,
) -> Result<Box<dyn BlobStore>, Sysexits> {
    match Url::parse(url.as_ref()) {
        Err(err) => {
            eprintln!("blobary: BLOBARY_URL is invalid: {}", err);
            Err(Sysexits::EX_DATAERR)
        }
        Ok(url) => match url.scheme() {
            "file" => open_store_from_file_url(url, writable),
            "memory" => open_store_from_memory_url(url, writable),
            #[cfg(feature = "redis")]
            "redis" => open_store_from_redis_url(url, writable),
            #[cfg(feature = "s3")]
            "s3" => open_store_from_s3_url(url, writable),
            #[cfg(feature = "sqlite")]
            "sqlite" => open_store_from_sqlite_url(url, writable),
            _ => {
                eprintln!("blobary: BLOBARY_URL has an unsupported URL scheme");
                Err(Sysexits::EX_DATAERR)
            }
        },
    }
}

fn open_store_from_file_url(url: Url, writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    match url.to_file_path() {
        Err(_) => {
            eprintln!("blobary: BLOBARY_URL contains an invalid path: {}", url);
            Err(Sysexits::EX_DATAERR)
        }
        Ok(path) => match DirectoryBlobStore::open_path(path, BlobStoreOptions { writable }) {
            Ok(store) => Ok(Box::new(store)),
            Err(err) => {
                eprintln!("blobary: {}", err);
                Err(Sysexits::EX_IOERR)
            }
        },
    }
}

fn open_store_from_memory_url(_url: Url, writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    Ok(Box::new(EphemeralBlobStore::new(BlobStoreOptions {
        writable,
    })))
}

#[cfg(feature = "redis")]
fn open_store_from_redis_url(url: Url, writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    match blobary::redis::RedisBlobStore::open(url, BlobStoreOptions { writable }) {
        Ok(store) => Ok(Box::new(store)),
        Err(err) => {
            eprintln!("blobary: {}", err);
            Err(Sysexits::EX_IOERR)
        }
    }
}

#[cfg(feature = "s3")]
fn open_store_from_s3_url(url: Url, writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    let url_path = url.path();
    let bucket_name = url.host_str().unwrap();
    let bucket_prefix = match url_path.chars().last() {
        None => "",
        Some('/') => &url_path[..url_path.len() - 1],
        _ => url_path,
    };
    match blobary::s3::S3BlobStore::open(bucket_name, bucket_prefix, BlobStoreOptions { writable })
    {
        Ok(store) => Ok(Box::new(store)),
        Err(err) => {
            eprintln!("blobary: {}", err);
            Err(Sysexits::EX_IOERR)
        }
    }
}

#[cfg(feature = "sqlite")]
fn open_store_from_sqlite_url(_url: Url, _writable: bool) -> Result<Box<dyn BlobStore>, Sysexits> {
    todo!("SQLite support not implemented yet") // TODO
}
