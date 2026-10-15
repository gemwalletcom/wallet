use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::SignerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasPriceType {
    Regular { gas_price: BigInt },
    Eip1559 { gas_price: BigInt, priority_fee: BigInt },
    Solana { gas_price: BigInt, priority_fee: BigInt, unit_price: BigInt },
}

impl GasPriceType {
    pub fn regular<T: Into<BigInt>>(gas_price: T) -> Self {
        Self::Regular { gas_price: gas_price.into() }
    }

    pub fn eip1559<T: Into<BigInt>, U: Into<BigInt>>(gas_price: T, priority_fee: U) -> Self {
        Self::Eip1559 {
            gas_price: gas_price.into(),
            priority_fee: priority_fee.into(),
        }
    }

    pub fn solana<T: Into<BigInt>, U: Into<BigInt>, V: Into<BigInt>>(gas_price: T, priority_fee: U, unit_price: V) -> Self {
        Self::Solana {
            gas_price: gas_price.into(),
            priority_fee: priority_fee.into(),
            unit_price: unit_price.into(),
        }
    }

    pub fn gas_price(&self) -> BigInt {
        match self {
            GasPriceType::Regular { gas_price } => gas_price.clone(),
            GasPriceType::Eip1559 { gas_price, .. } => gas_price.clone(),
            GasPriceType::Solana { gas_price, .. } => gas_price.clone(),
        }
    }

    pub fn gas_price_u64(&self) -> Result<u64, SignerError> {
        self.gas_price().to_u64().ok_or_else(|| SignerError::invalid_input("invalid gas price"))
    }

    pub fn priority_fee(&self) -> BigInt {
        match self {
            GasPriceType::Regular { .. } => BigInt::from(0),
            GasPriceType::Eip1559 { priority_fee, .. } => priority_fee.clone(),
            GasPriceType::Solana { priority_fee, .. } => priority_fee.clone(),
        }
    }

    pub fn priority_fee_u64(&self) -> Result<u64, SignerError> {
        self.priority_fee().to_u64().ok_or_else(|| SignerError::invalid_input("invalid priority fee"))
    }

    pub fn unit_price(&self) -> BigInt {
        match self {
            GasPriceType::Regular { .. } => BigInt::from(0),
            GasPriceType::Eip1559 { .. } => BigInt::from(0),
            GasPriceType::Solana { unit_price, .. } => unit_price.clone(),
        }
    }

    pub fn unit_price_u64(&self) -> Result<u64, SignerError> {
        self.unit_price().to_u64().ok_or_else(|| SignerError::invalid_input("invalid unit price"))
    }

    pub fn total_fee(&self) -> BigInt {
        self.gas_price() + self.priority_fee()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_price() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.gas_price(), BigInt::from(1000u64));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.gas_price(), BigInt::from(2000u64));
    }

    #[test]
    fn test_u64_accessors() {
        for value in [0, 1000, u64::MAX] {
            let gas_price = GasPriceType::solana(value, value, value);
            assert_eq!(gas_price.gas_price_u64().unwrap(), value);
            assert_eq!(gas_price.priority_fee_u64().unwrap(), value);
            assert_eq!(gas_price.unit_price_u64().unwrap(), value);
        }
        for value in [BigInt::from(-1), BigInt::from(u64::MAX) + 1] {
            let gas_price = GasPriceType::solana(value.clone(), value.clone(), value);
            assert_eq!(gas_price.gas_price_u64(), Err(SignerError::invalid_input("invalid gas price")));
            assert_eq!(gas_price.priority_fee_u64(), Err(SignerError::invalid_input("invalid priority fee")));
            assert_eq!(gas_price.unit_price_u64(), Err(SignerError::invalid_input("invalid unit price")));
        }
    }

    #[test]
    fn priority_fee() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.priority_fee(), BigInt::from(0));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.priority_fee(), BigInt::from(500u64));
    }

    #[test]
    fn unit_price() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.unit_price(), BigInt::from(0));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.unit_price(), BigInt::from(0));

        let solana = GasPriceType::solana(BigInt::from(5000u64), BigInt::from(1000u64), BigInt::from(200u64));
        assert_eq!(solana.unit_price(), BigInt::from(200u64));
    }

    #[test]
    fn total_fee() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.total_fee(), BigInt::from(1000u64));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.total_fee(), BigInt::from(2500u64));

        let solana = GasPriceType::solana(BigInt::from(5000u64), BigInt::from(1000u64), BigInt::from(200u64));
        assert_eq!(solana.total_fee(), BigInt::from(6000u64));
    }
}
