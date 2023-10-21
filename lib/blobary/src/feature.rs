// This is free and unencumbered software released into the public domain.

#[allow(unused)]
pub static FEATURES: &'static [&str] = &[
    #[cfg(feature = "base58")]
    "base58",
    #[cfg(feature = "encrypt")]
    "encrypt",
    #[cfg(feature = "gzip")]
    "gzip",
    #[cfg(feature = "lz4")]
    "lz4",
    #[cfg(feature = "magic")]
    "magic",
    #[cfg(feature = "redis")]
    "redis",
    #[cfg(feature = "sqlite")]
    "sqlite",
    #[cfg(feature = "tracing")]
    "tracing",
    #[cfg(feature = "zeroize")]
    "zeroize",
];
