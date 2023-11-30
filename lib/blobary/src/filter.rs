// This is free and unencumbered software released into the public domain.

use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Read, Result, Write},
};

/// A filter is a function that transforms a blob.
pub trait Filter {
    fn encode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64>;

    fn decode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64>;
}

impl Clone for Box<dyn Filter> {
    fn clone(&self) -> Self {
        unimplemented!("Clone is not implemented for dyn Filter")
    }
}

impl Debug for dyn Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Filter {{ ... }}")
    }
}

pub struct NoopFilter {}

impl Filter for NoopFilter {
    fn encode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64> {
        io::copy(input, output)
    }

    fn decode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64> {
        io::copy(input, output)
    }
}
