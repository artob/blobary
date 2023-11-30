// This is free and unencumbered software released into the public domain.

use crate::Filter;
use std::io::{self, Read, Result, Write};

#[cfg(feature = "gzip")]
pub struct GzipCompressor {}

#[cfg(feature = "gzip")]
impl Filter for GzipCompressor {
    fn encode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64> {
        use libflate::gzip;
        let header = gzip::HeaderBuilder::new().finish(); // TODO: set header options
        let options = gzip::EncodeOptions::new().header(header);
        let mut encoder = gzip::Encoder::with_options(output, options)?;
        let result = io::copy(input, &mut encoder)?;
        let _ = encoder.finish().into_result()?;
        Ok(result)
    }

    fn decode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64> {
        use libflate::gzip;
        let mut decoder = gzip::Decoder::new(input)?;
        let result = io::copy(&mut decoder, output)?;
        Ok(result)
    }
}

#[cfg(feature = "lz4")]
pub struct Lz4Compressor {}

#[cfg(feature = "lz4")]
impl Filter for Lz4Compressor {
    fn encode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64> {
        use lz4_flex::frame;
        let mut encoder = frame::FrameEncoder::new(output);
        let result = io::copy(input, &mut encoder)?;
        let _ = encoder.finish()?;
        Ok(result)
    }

    fn decode(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<u64> {
        use lz4_flex::frame;
        let mut decoder = frame::FrameDecoder::new(input);
        let result = io::copy(&mut decoder, output)?;
        Ok(result)
    }
}
