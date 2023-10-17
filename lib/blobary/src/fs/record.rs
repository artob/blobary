// This is free and unencumbered software released into the public domain.

use std::mem::size_of;
use zerocopy::{byteorder::network_endian::U64, AsBytes};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes, Unaligned};
use zeroize::Zeroize;

pub const RECORD_SIZE: usize = size_of::<PersistentBlobRecord>();

#[derive(Debug, Copy, Clone, FromZeroes, FromBytes, AsBytes, Unaligned)]
#[repr(C)]
pub(crate) struct PersistentBlobRecord(pub [u8; 32], pub U64);

const _: () = assert!(
    size_of::<PersistentBlobRecord>() == 40,
    "sizeof(PersistentBlobRecord) == 40"
);

impl Zeroize for PersistentBlobRecord {
    fn zeroize(&mut self) {
        self.as_bytes_mut().zeroize();
    }
}
