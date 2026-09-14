use sha2::{Digest, Sha256};

fn namespaced_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(b":");
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    let mut data = [0u8; 8];
    data.copy_from_slice(&hash[..8]);
    data
}

pub fn global_discriminator(name: &str) -> [u8; 8] {
    namespaced_discriminator("global", name)
}

#[cfg(test)]
mod tests {
    use hex_lit::hex;

    use super::*;

    #[test]
    fn test_global_discriminator() {
        assert_eq!(global_discriminator("init_order"), hex!("204c290c27a284db"));
    }
}
