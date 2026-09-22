use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    #[serde(rename = "self")]
    SelfTransfer,
    Outgoing,
    Incoming,
}

impl TransactionDirection {
    pub fn from_parties(from: &str, to: &str, addresses: &[String]) -> Self {
        let contains = |address: &str| !address.is_empty() && addresses.iter().any(|candidate| candidate.eq_ignore_ascii_case(address));
        match (contains(from), contains(to)) {
            (true, true) => Self::SelfTransfer,
            (true, false) => Self::Outgoing,
            (false, true) => Self::Incoming,
            (false, false) => Self::SelfTransfer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_parties() {
        let user = "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".to_string();
        let contract = "0x54914A963c4197172130C26D496a367bD6609D88".to_string();

        assert_eq!(TransactionDirection::from_parties(&contract, &user, &[user.to_lowercase()]), TransactionDirection::Incoming);
        assert_eq!(TransactionDirection::from_parties(&user, &contract, &[user.to_lowercase()]), TransactionDirection::Outgoing);
        assert_eq!(TransactionDirection::from_parties(&user, &user, std::slice::from_ref(&user)), TransactionDirection::SelfTransfer);
        assert_eq!(TransactionDirection::from_parties(&user, &user.to_lowercase(), std::slice::from_ref(&user)), TransactionDirection::SelfTransfer);
        assert_eq!(TransactionDirection::from_parties(&contract, &user, &[]), TransactionDirection::SelfTransfer);
        assert_eq!(TransactionDirection::from_parties(&contract, &user, &[contract.clone(), user.clone()]), TransactionDirection::SelfTransfer);
    }
}
