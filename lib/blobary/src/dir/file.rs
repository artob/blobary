// This is free and unencumbered software released into the public domain.

use std::{
    io::{Read, Result, Seek, Write},
    os::unix::prelude::FileExt,
};

pub trait File: Seek + Read + Write + FileSync + FileExt + Send {}

impl File for std::fs::File {}

impl File for cap_std::fs::File {}

pub trait FileSync {
    fn sync_all(&self) -> Result<()>;
    fn sync_data(&self) -> Result<()>;
}

impl FileSync for std::fs::File {
    fn sync_all(&self) -> Result<()> {
        std::fs::File::sync_all(self)
    }

    fn sync_data(&self) -> Result<()> {
        std::fs::File::sync_data(self)
    }
}

impl FileSync for cap_std::fs::File {
    fn sync_all(&self) -> Result<()> {
        cap_std::fs::File::sync_all(self)
    }

    fn sync_data(&self) -> Result<()> {
        cap_std::fs::File::sync_data(self)
    }
}
