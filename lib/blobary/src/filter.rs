// This is free and unencumbered software released into the public domain.

use std::io::{self, Read, Result, Write};

/// A filter is a function that transforms a blob.
pub trait Filter {
    fn encode(&self, input: &mut impl Read, output: &mut impl Write) -> Result<u64>;
    fn decode(&self, input: &mut impl Read, output: &mut impl Write) -> Result<u64>;
}

pub struct NoopFilter {}

impl Filter for NoopFilter {
    fn encode(&self, input: &mut impl Read, output: &mut impl Write) -> Result<u64> {
        io::copy(input, output)
    }

    fn decode(&self, input: &mut impl Read, output: &mut impl Write) -> Result<u64> {
        io::copy(input, output)
    }
}
