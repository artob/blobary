// This is free and unencumbered software released into the public domain.

use std::{
    fs::File,
    io::{stdin, Read, Result},
    path::Path,
};

pub fn list_inputs(input_paths: &Vec<impl AsRef<Path>>) -> Result<Vec<String>> {
    if input_paths.is_empty() {
        Ok(vec![String::from("/dev/stdin")])
    } else {
        let mut inputs = Vec::with_capacity(input_paths.len());
        for input_path in input_paths {
            let input_path = String::from(input_path.as_ref().to_string_lossy());
            inputs.push(input_path);
        }
        Ok(inputs)
    }
}

pub fn open_inputs(input_paths: &Vec<impl AsRef<Path>>) -> Result<Vec<(String, Box<dyn Read>)>> {
    if input_paths.is_empty() {
        Ok(vec![(String::from("/dev/stdin"), Box::new(stdin()))])
    } else {
        let mut inputs = Vec::with_capacity(input_paths.len());
        for input_path in input_paths {
            let input: Box<dyn Read> = Box::new(File::open(input_path)?);
            let input_path = String::from(input_path.as_ref().to_string_lossy());
            inputs.push((input_path, input));
        }
        Ok(inputs)
    }
}
