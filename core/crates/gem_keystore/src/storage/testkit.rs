use super::{
    constants::{ARGON2_SALT_LEN, DEFAULT_ARGON2_ITERATIONS, DEFAULT_ARGON2_MEMORY_KIB, DEFAULT_ARGON2_OUTPUT_LEN, DEFAULT_ARGON2_PARALLELISM},
    format::{CipherV4, CryptoV4, FileV4, KdfV4},
    types::KdfParams,
};

const MOCK_ARGON2_MEMORY_KIB: u32 = 64;
const MOCK_ARGON2_ITERATIONS: u32 = 1;
const MOCK_ARGON2_PARALLELISM: u32 = 1;
const MOCK_ARGON2_SALT_BYTE: u8 = 7;

impl KdfParams {
    pub(super) fn mock() -> Self {
        Self::Argon2id {
            memory_kib: MOCK_ARGON2_MEMORY_KIB,
            iterations: MOCK_ARGON2_ITERATIONS,
            parallelism: MOCK_ARGON2_PARALLELISM,
            salt: [MOCK_ARGON2_SALT_BYTE; ARGON2_SALT_LEN],
            output_len: DEFAULT_ARGON2_OUTPUT_LEN,
        }
    }
}

impl FileV4 {
    pub(super) fn mock() -> Self {
        Self {
            version: 4,
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            kind: "mnemonic".to_string(),
            crypto: CryptoV4 {
                kdf: KdfV4 {
                    algorithm: "argon2id".to_string(),
                    memory_kib: DEFAULT_ARGON2_MEMORY_KIB,
                    iterations: DEFAULT_ARGON2_ITERATIONS,
                    parallelism: DEFAULT_ARGON2_PARALLELISM,
                    salt: "00000000000000000000000000000000".to_string(),
                    output_len: 32,
                },
                cipher: CipherV4 {
                    algorithm: "aes-256-gcm".to_string(),
                    nonce: "000000000000000000000000".to_string(),
                    tag_len: 16,
                },
                ciphertext: "00000000000000000000000000000000".to_string(),
            },
        }
    }
}
