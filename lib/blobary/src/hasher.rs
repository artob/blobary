// This is free and unencumbered software released into the public domain.

use crate::BlobHash;

pub type BlobHasher = blake3::Hasher;

pub fn hash(input: impl AsRef<[u8]>) -> BlobHash {
    blake3::hash(input.as_ref())
}
