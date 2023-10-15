// This is free and unencumbered software released into the public domain.

use std::io::{Cursor, Read, Seek, Write};

/// A blob's locally-unique integer ID in a [`BlobStore`].
#[allow(unused)]
pub type BlobID = usize;

/// The blob's globally-unique cryptographic BLAKE3 hash.
#[allow(unused)]
pub type BlobHash = blake3::Hash;

/// A blob is a unique byte sequence of data.
pub trait Blob: Seek + Read {}

/// A mutable blob is a unique byte sequence of data.
pub trait BlobMut: Blob + Write {}

impl Blob for Cursor<&[u8]> {}

impl Blob for Cursor<Vec<u8>> {}

impl Blob for std::fs::File {}

impl BlobMut for Cursor<Vec<u8>> {}

impl BlobMut for std::fs::File {}
