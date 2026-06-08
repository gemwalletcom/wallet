pub mod compare;
mod error;

#[cfg(feature = "hash")]
pub mod hash;
#[cfg(feature = "pbkdf")]
pub mod pbkdf;
#[cfg(feature = "random")]
pub mod random;

pub use error::CryptoError;
