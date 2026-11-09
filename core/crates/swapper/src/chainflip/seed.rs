use gem_crypto::random;

use crate::SwapperError;

pub fn generate_random_seed(max_bytes: usize) -> Result<Vec<u8>, SwapperError> {
    if max_bytes == 0 || max_bytes > 32 {
        return Ok(vec![]);
    }
    let len = random::index(max_bytes)? + 1;
    Ok(random::bytes::<32>()?[..len].to_vec())
}
