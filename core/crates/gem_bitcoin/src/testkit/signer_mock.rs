use num_bigint::BigInt;
use num_bigint::BigUint;
use primitives::{Asset, AssetId, BitcoinChain, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO};

use crate::testkit::address_mock::{mock_destination_address, mock_p2wpkh_address, mock_sender_address};

pub use primitives::testkit::{signer_mock::TEST_PRIVATE_KEY, zcash_mock::TEST_ZCASH_BRANCH_ID};

pub const TEST_UTXO_TXID: &str = "0000000000000000000000000000000000000000000000000000000000000001";

pub fn mock_transfer_input(chain: BitcoinChain) -> SignerInput {
    let sender_address = mock_sender_address(chain);
    let destination_address = mock_destination_address(chain);
    mock_transfer_input_with_utxos(chain, &sender_address, &destination_address, "10000", vec![mock_utxo_with_address(&sender_address)])
}

pub fn mock_transfer_input_with_utxos(chain: BitcoinChain, sender_address: &str, destination_address: &str, value: &str, utxos: Vec<UTXO>) -> SignerInput {
    let metadata = match chain {
        BitcoinChain::Zcash => TransactionLoadMetadata::Zcash {
            branch_id: TEST_ZCASH_BRANCH_ID.to_string(),
            utxos,
        },
        _ => TransactionLoadMetadata::Bitcoin { utxos },
    };

    SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer { asset: Asset::from_chain(chain.get_chain()) },
            sender_address: sender_address.to_string(),
            destination_address: destination_address.to_string(),
            value: value.parse().unwrap(),
            gas_price: GasPriceType::regular(BigInt::from(10u64)),
            memo: None,
            is_max_value: false,
            metadata,
        },
        TransactionFee::new_from_fee(BigInt::from(10u64), AssetId::from_chain(Chain::Bitcoin)),
    )
}

pub fn mock_p2wpkh_transfer_input() -> SignerInput {
    let address = mock_p2wpkh_address();
    mock_transfer_input_with_utxos(BitcoinChain::Bitcoin, &address, &address, "10000", vec![mock_utxo_with(TEST_UTXO_TXID, 0, "50000", &address)])
}

pub fn mock_funded_transfer_input(chain: BitcoinChain) -> SignerInput {
    let mut input = mock_transfer_input(chain);
    match chain {
        BitcoinChain::Doge => input.input.value = BigUint::from(1_000_000u64),
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Dash | BitcoinChain::Zcash => {}
    }
    match &mut input.input.metadata {
        TransactionLoadMetadata::Bitcoin { utxos } | TransactionLoadMetadata::Zcash { utxos, .. } => {
            utxos[0].value = BigUint::from(100_000_000u64);
        }
        _ => {}
    }
    input
}

pub fn mock_utxo_with(transaction_id: &str, vout: i32, value: &str, address: &str) -> UTXO {
    UTXO {
        transaction_id: transaction_id.to_string(),
        vout,
        value: value.parse().unwrap(),
        address: address.to_string(),
    }
}

pub fn mock_utxo_with_address(address: &str) -> UTXO {
    mock_utxo_with(TEST_UTXO_TXID, 0, "50000", address)
}
