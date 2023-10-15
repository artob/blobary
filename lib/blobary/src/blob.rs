// This is free and unencumbered software released into the public domain.

use std::io::{Cursor, Read, Seek, Write};

#[allow(unused)]
pub type BlobID = usize;

#[allow(unused)]
pub type BlobHash = blake3::Hash;

pub trait Blob: Seek + Read {}

pub trait BlobMut: Blob + Write {}

impl Blob for Cursor<&[u8]> {}

impl Blob for Cursor<Vec<u8>> {}

impl Blob for std::fs::File {}

impl BlobMut for Cursor<Vec<u8>> {}

impl BlobMut for std::fs::File {}
