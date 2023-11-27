// This is free and unencumbered software released into the public domain.

use crate::Filter;
use std::io::{self, Read, Result};

#[cfg(feature = "gzip")]
pub struct GzipCompressor {}

#[cfg(feature = "gzip")]
impl Filter for GzipCompressor {
    fn encode(&self, input: &mut impl Read) -> Result<Vec<u8>> {
        use libflate::gzip;
        let header = gzip::HeaderBuilder::new().finish(); // TODO: set header options
        let options = gzip::EncodeOptions::new().header(header);
        let mut encoder = gzip::Encoder::with_options(Vec::new(), options)?;
        io::copy(input, &mut encoder)?;
        Ok(encoder.finish().into_result()?)
    }

    fn decode(&self, input: &mut impl Read) -> Result<Vec<u8>> {
        use libflate::gzip;
        let mut decoder = gzip::Decoder::new(input)?;
        let mut output = Vec::new();
        io::copy(&mut decoder, &mut output)?;
        Ok(output)
    }
}

#[cfg(feature = "lz4")]
pub struct Lz4Compressor {}

#[cfg(feature = "lz4")]
impl Filter for Lz4Compressor {
    fn encode(&self, input: &mut impl Read) -> Result<Vec<u8>> {
        use lz4_flex::frame;
        let output = Vec::new();
        let mut encoder = frame::FrameEncoder::new(output);
        io::copy(input, &mut encoder)?;
        Ok(encoder.finish()?)
    }

    fn decode(&self, input: &mut impl Read) -> Result<Vec<u8>> {
        use lz4_flex::frame;
        let mut output = Vec::new();
        let mut decoder = frame::FrameDecoder::new(input);
        io::copy(&mut decoder, &mut output)?;
        Ok(output)
    }
}
