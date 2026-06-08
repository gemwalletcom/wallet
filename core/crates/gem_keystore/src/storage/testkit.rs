use super::{
    constants::{ARGON2_SALT_LEN, DEFAULT_ARGON2_OUTPUT_LEN},
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
