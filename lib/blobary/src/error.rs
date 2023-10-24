// This is free and unencumbered software released into the public domain.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlobHashError {
    #[error("empty blob hash")]
    Empty,
    #[error("invalid blob hash length: {0} != 32")]
    InvalidLength(usize),
    #[error("invalid blob hash: {0}")]
    InvalidInput(String),
}

#[derive(Error, Debug)]
pub enum BlobStoreError {
    #[error("unsupported operation")]
    Unsupported,
    #[error("unimplemented operation: {0}")]
    Unimplemented(String),
    #[error("unexpected error")]
    Unexpected,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

#[cfg(feature = "redis")]
impl From<redis::RedisError> for BlobStoreError {
    fn from(error: redis::RedisError) -> Self {
        Self::Other(Box::new(error))
    }
}

#[cfg(feature = "s3")]
impl From<s3::creds::error::CredentialsError> for BlobStoreError {
    fn from(error: s3::creds::error::CredentialsError) -> Self {
        use s3::creds::error::CredentialsError::*;
        match error {
            Io(error) => Self::IO(error),
            _ => Self::Other(Box::new(error)),
        }
    }
}

#[cfg(feature = "s3")]
impl From<s3::error::S3Error> for BlobStoreError {
    fn from(error: s3::error::S3Error) -> Self {
        use s3::error::S3Error::*;
        match error {
            Io(error) => Self::IO(error),
            _ => Self::Other(Box::new(error)),
        }
    }
}
