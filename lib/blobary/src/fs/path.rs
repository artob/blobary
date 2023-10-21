// This is free and unencumbered software released into the public domain.

use crate::BlobHash;
use std::path::PathBuf;

pub fn encode_into_path(blob_hash: BlobHash) -> PathBuf {
    PathBuf::from(blob_hash.to_hex().as_str())
}
