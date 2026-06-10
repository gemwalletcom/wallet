use zeroize::Zeroizing;

pub(super) const PRIVATE_KEY_LEN: usize = 32;
pub(super) const CHAIN_CODE_LEN: usize = 32;

pub(super) struct DerivedChild {
    pub(super) private_key: Zeroizing<[u8; PRIVATE_KEY_LEN]>,
    pub(super) chain_code: Zeroizing<[u8; CHAIN_CODE_LEN]>,
}

pub(super) fn split_left_key(output: &[u8; 64]) -> [u8; PRIVATE_KEY_LEN] {
    let mut key = [0u8; PRIVATE_KEY_LEN];
    key.copy_from_slice(&output[..PRIVATE_KEY_LEN]);
    key
}

pub(super) fn split_right_chain_code(output: &[u8; 64]) -> [u8; CHAIN_CODE_LEN] {
    let mut chain_code = [0u8; CHAIN_CODE_LEN];
    chain_code.copy_from_slice(&output[PRIVATE_KEY_LEN..]);
    chain_code
}
