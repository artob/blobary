// This is free and unencumbered software released into the public domain.

use std::{
    fs::File,
    io::{stdout, Result, Write},
    path::Path,
};

pub fn open_output(output_path: &Option<impl AsRef<Path>>) -> Result<Box<dyn Write>> {
    let output: Box<dyn Write> = if output_path.is_none() {
        Box::new(stdout()) // /dev/stdout
    } else {
        Box::new(File::create(output_path.as_ref().unwrap()).unwrap())
    };
    Ok(output)
}
