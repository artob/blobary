// This is free and unencumbered software released into the public domain.

use blobary::BlobHash;

pub fn parse_hash(input: &str) -> std::io::Result<BlobHash> {
    decode_hash(input)
}

pub fn decode_hash(blob_hash_str: impl AsRef<str>) -> std::io::Result<BlobHash> {
    let blob_hash_str = blob_hash_str.as_ref();
    if let Ok(blob_hash) = BlobHash::from_hex(blob_hash_str) {
        return Ok(blob_hash);
    }
    #[cfg(feature = "base58")]
    if let Ok(blob_hash) = bs58::decode(blob_hash_str).into_vec() {
        return Ok(BlobHash::from_bytes(
            blob_hash.as_slice().try_into().unwrap(),
        ));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("invalid blob hash: {}", blob_hash_str),
    ))
}

pub fn encode_hash(blob_hash: BlobHash) -> String {
    #[cfg(not(feature = "base58"))]
    return blob_hash.to_hex().to_string();
    #[cfg(feature = "base58")]
    return bs58::encode(blob_hash.0.as_bytes()).into_string();
}
