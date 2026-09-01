use num_bigint::BigInt;
use num_bigint::BigUint;

use crate::{Asset, AssetId, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO};

pub const TEST_ZCASH_BRANCH_ID: &str = "4dec4df0";

pub fn mock_signer_input(sender_address: String, destination_address: String) -> SignerInput {
    SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Zcash)),
            sender_address: sender_address.clone(),
            destination_address,
            value: BigUint::from(20000u64),
            gas_price: GasPriceType::regular(BigInt::from(1u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Zcash {
                branch_id: TEST_ZCASH_BRANCH_ID.to_string(),
                utxos: vec![UTXO {
                    transaction_id: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                    vout: 0,
                    value: BigUint::from(50000u64),
                    address: sender_address,
                }],
            },
        },
        TransactionFee::new_from_fee(BigInt::from(1u64), AssetId::from_chain(Chain::Zcash)),
    )
}
