use num_bigint::BigInt;
use num_bigint::BigUint;

use crate::{Asset, AssetId, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO};

pub const TEST_ZCASH_TRANSPARENT_ADDRESS: &str = "t1VmmGiyjVNeCjxDZzg7vZmd99WyzVby9yC";
pub const TEST_ZCASH_TEX_ADDRESS: &str = "tex1s2rt77ggv6q989lr49rkgzmh5slsksa9khdgte";
pub const TEST_ZCASH_UNIFIED_ADDRESS: &str = "u1qhwq6evpqq04ga6dn06mexlk6pcarhru4mhpkv63ar7nsmqvk5057lgtuj2yx6nhxlu0wuhzuw74v4q0eskt7gxy2xplq4qe5m560gukxtnx86mu5qljhgygyxzy7ulavg02qc42hxs";

pub const TEST_ZCASH_BRANCH_ID: &str = "4dec4df0";
pub const TEST_ZCASH_NU6_3_BRANCH_ID: &str = "37a5165b";

pub fn mock_signer_input(sender_address: String, destination_address: String) -> SignerInput {
    mock_signer_input_with_branch_id(sender_address, destination_address, TEST_ZCASH_BRANCH_ID)
}

pub fn mock_signer_input_with_branch_id(sender_address: String, destination_address: String, branch_id: &str) -> SignerInput {
    SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Zcash) },
            sender_address: sender_address.clone(),
            destination_address,
            value: BigUint::from(20000u64),
            gas_price: GasPriceType::regular(BigInt::from(1u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Zcash {
                branch_id: branch_id.to_string(),
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
