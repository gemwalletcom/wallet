use zeroize::Zeroizing;

use crate::CryptoError;
use crate::hash::hmac_sha512;

const HMAC_SHA512_OUTPUT_LEN: usize = 64;

pub fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], iterations: usize, output_len: usize) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if iterations == 0 {
        return Err(CryptoError::invalid_input("PBKDF2 iterations must be greater than zero"));
    }

    let mut output = Zeroizing::new(vec![0u8; output_len]);
    for (block_index, chunk) in output.chunks_mut(HMAC_SHA512_OUTPUT_LEN).enumerate() {
        let mut block_salt = Zeroizing::new(Vec::with_capacity(salt.len() + 4));
        block_salt.extend_from_slice(salt);
        let block_number = u32::try_from(block_index + 1).map_err(|_| CryptoError::invalid_input("PBKDF2 output length is too large"))?;
        block_salt.extend_from_slice(&block_number.to_be_bytes());

        let mut u = hmac_sha512(password, block_salt.as_slice())?;
        let mut block = Zeroizing::new(*u);
        for _ in 1..iterations {
            u = hmac_sha512(password, u.as_slice())?;
            for (left, right) in block.iter_mut().zip(u.iter()) {
                *left ^= *right;
            }
        }
        chunk.copy_from_slice(&block[..chunk.len()]);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_hmac_sha512_vector() {
        let output = pbkdf2_hmac_sha512(b"password", b"salt", 1, HMAC_SHA512_OUTPUT_LEN).unwrap();

        assert_eq!(
            hex::encode(output.as_slice()),
            "867f70cf1ade02cff3752599a3a53dc4af34c7a669815ae5d513554e1c8cf252\
             c02d470a285a0501bad999bfe943c08f050235d7d68b1da55e63f73b60a57fce"
                .replace(char::is_whitespace, "")
        );
    }

    #[test]
    fn test_pbkdf2_hmac_sha512_rejects_zero_iterations() {
        assert_eq!(
            pbkdf2_hmac_sha512(b"password", b"salt", 0, HMAC_SHA512_OUTPUT_LEN).unwrap_err(),
            CryptoError::invalid_input("PBKDF2 iterations must be greater than zero")
        );
    }
}
