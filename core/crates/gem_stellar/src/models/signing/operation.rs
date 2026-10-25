use super::StellarAssetData;
use crate::address::StellarAddress;

#[derive(Clone)]
pub enum Memo {
    None,
    Text(String),
    #[cfg_attr(not(test), allow(unused))]
    Id(u64),
}

#[derive(Clone)]
pub enum Operation {
    CreateAccount {
        destination: StellarAddress,
        amount: u64,
    },
    Payment {
        destination: StellarAddress,
        asset: Option<StellarAssetData>,
        amount: u64,
    },
    ChangeTrust {
        asset: StellarAssetData,
    },
    PathPaymentStrictSend {
        send_asset: Option<StellarAssetData>,
        send_amount: u64,
        destination: StellarAddress,
        destination_asset: Option<StellarAssetData>,
        destination_min: u64,
        path: Vec<Option<StellarAssetData>>,
    },
}

impl Operation {
    pub fn operation_type(&self) -> u32 {
        match self {
            Self::CreateAccount { .. } => 0,
            Self::Payment { .. } => 1,
            Self::ChangeTrust { .. } => 6,
            Self::PathPaymentStrictSend { .. } => 13,
        }
    }
}
