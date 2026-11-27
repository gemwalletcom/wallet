use crate::CryptoError;

pub fn fill(bytes: &mut [u8]) -> Result<(), CryptoError> {
    getrandom::fill(bytes).map_err(|_| CryptoError::Random)
}

pub fn bytes<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut bytes = [0u8; N];
    fill(&mut bytes)?;
    Ok(bytes)
}

pub fn index(len: usize) -> Result<usize, CryptoError> {
    if len == 0 {
        return Err(CryptoError::InvalidInput("random index needs a non-empty range".to_string()));
    }
    let len = len as u64;
    let accepted = u64::MAX - u64::MAX % len;
    loop {
        let value = u64::from_le_bytes(bytes::<8>()?);
        if value < accepted {
            return Ok((value % len) as usize);
        }
    }
}

pub fn shuffle<T>(items: &mut [T]) -> Result<(), CryptoError> {
    for upper in (1..items.len()).rev() {
        items.swap(upper, index(upper + 1)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_fill() {
        let mut first = [0u8; 32];
        let mut second = [0u8; 32];
        assert_eq!(fill(&mut first), Ok(()));
        assert_eq!(fill(&mut second), Ok(()));
        assert_ne!(first, second);
        assert_ne!(first, [0u8; 32]);
    }

    #[test]
    fn test_index() {
        assert_eq!(index(0), Err(CryptoError::InvalidInput("random index needs a non-empty range".to_string())));
        assert_eq!(index(1), Ok(0));

        let drawn: HashSet<usize> = (0..1_000).map(|_| index(5).unwrap()).collect();
        assert_eq!(drawn, (0..5).collect());
    }

    #[test]
    fn test_shuffle() {
        let mut empty: [u8; 0] = [];
        assert_eq!(shuffle(&mut empty), Ok(()));

        let orders: HashSet<[u8; 4]> = (0..2_000)
            .map(|_| {
                let mut items = [1, 2, 3, 4];
                shuffle(&mut items).unwrap();
                items
            })
            .collect();
        assert_eq!(orders.len(), 24);
    }
}
