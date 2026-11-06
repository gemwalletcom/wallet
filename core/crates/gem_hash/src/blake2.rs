use blake2::digest::CustomizedInit;
use blake2::digest::consts::{U28, U32, U64};
use blake2::{Blake2b, Digest};

pub fn blake2b_224(bytes: &[u8]) -> [u8; 28] {
    Blake2b::<U28>::digest(bytes).into()
}

pub fn blake2b_256(bytes: &[u8]) -> [u8; 32] {
    Blake2b::<U32>::digest(bytes).into()
}

pub fn blake2b_256_personal(bytes: &[u8], personal: &[u8; 16]) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new_customized(personal);
    hasher.update(bytes);
    hasher.finalize().into()
}

pub fn blake2b_512(bytes: &[u8]) -> [u8; 64] {
    Blake2b::<U64>::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2b_256_personal() {
        assert_eq!(gem_encoding::encode_hex(blake2b_256_personal(&[], b"ZTxIdSaplingHash")), "6f2fc8f98feafd94e74a0df4bed74391ee0b5a69945e4ced8ca8a095206f00ae");
    }
}
