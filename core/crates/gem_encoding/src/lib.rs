mod error;
mod percent_encoding;
#[cfg(feature = "protobuf")]
pub mod protobuf;

#[cfg(feature = "base32")]
mod base32;
#[cfg(feature = "base64")]
mod base64;
#[cfg(any(feature = "base32", feature = "base64"))]
mod base_n;

pub use error::{EncodingError, EncodingType};
pub use percent_encoding::{is_valid_percent_encoding, percent_decode_lossy, percent_encode};

#[cfg(feature = "base32")]
pub use crate::base32::{decode_base32, encode_base32};
#[cfg(feature = "base64")]
pub use crate::base64::{decode_base64, decode_base64_no_pad, decode_base64_url, encode_base64, encode_base64_url};
