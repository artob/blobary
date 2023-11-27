// This is free and unencumbered software released into the public domain.

use std::io::{Read, Result};

/// A filter is a function that transforms a blob.
pub trait Filter {
    fn encode(&self, input: &mut impl Read) -> Result<Vec<u8>>;
    fn decode(&self, input: &mut impl Read) -> Result<Vec<u8>>;
}
