#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignatureBytes([u8; 64]);

impl Default for SignatureBytes {
    fn default() -> Self {
        Self([0; 64])
    }
}

impl SignatureBytes {
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}
