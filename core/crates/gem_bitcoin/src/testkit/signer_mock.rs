use num_bigint::BigInt;
use primitives::{
    Asset, BitcoinChain, GasPriceType, SignerInput, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO,
    swap::{SwapData, SwapProviderData, SwapQuote, SwapQuoteData},
};

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
            input_type: TransactionInputType::Transfer(Asset::from_chain(chain.get_chain())),
            sender_address: sender_address.to_string(),
            destination_address: destination_address.to_string(),
            value: value.to_string(),
            gas_price: GasPriceType::regular(BigInt::from(1u64)),
            memo: None,
            is_max_value: false,
            metadata,
        },
        TransactionFee::new_from_fee(BigInt::from(1u64)),
    )
}

pub fn mock_p2wpkh_transfer_input() -> SignerInput {
    let address = mock_p2wpkh_address();
    mock_transfer_input_with_utxos(
        BitcoinChain::Bitcoin,
        &address,
        &address,
        "10000",
        vec![mock_utxo_with(TEST_UTXO_TXID, 0, "50000", &address)],
    )
}

pub fn mock_funded_transfer_input(chain: BitcoinChain) -> SignerInput {
    let mut input = mock_transfer_input(chain);
    match &mut input.input.metadata {
        TransactionLoadMetadata::Bitcoin { utxos } | TransactionLoadMetadata::Zcash { utxos, .. } => {
            utxos[0].value = "100000000".to_string();
        }
        _ => {}
    }
    input
}

pub fn mock_transfer_swap_input(chain: BitcoinChain, memo: &str) -> SignerInput {
    mock_swap_input(chain, SwapProvider::Thorchain, Some(false), |destination_address, value| {
        SwapQuoteData::new_transfer(destination_address, value, Some(memo.to_string()))
    })
}

pub fn mock_contract_swap_input(chain: BitcoinChain, nulldata_hex: &str, use_max_amount: bool) -> SignerInput {
    mock_swap_input(chain, SwapProvider::Chainflip, Some(use_max_amount), |destination_address, value| {
        SwapQuoteData::new_contract(destination_address, value, nulldata_hex.to_string(), None, None)
    })
}

fn mock_swap_input(chain: BitcoinChain, provider: SwapProvider, use_max_amount: Option<bool>, quote_data: impl FnOnce(String, String) -> SwapQuoteData) -> SignerInput {
    let mut input = mock_funded_transfer_input(chain);
    let sender_address = input.sender_address.clone();
    let destination_address = input.destination_address.clone();
    let value = input.value.clone();

    input.input.input_type = TransactionInputType::Swap(
        Asset::from_chain(chain.get_chain()),
        Asset::from_chain(BitcoinChain::Bitcoin.get_chain()),
        SwapData {
            quote: SwapQuote {
                from_address: sender_address,
                from_value: value.clone(),
                min_from_value: None,
                to_address: destination_address.clone(),
                to_value: value.clone(),
                provider_data: SwapProviderData {
                    provider,
                    name: provider.name().to_string(),
                    protocol_name: provider.protocol_name().to_string(),
                },
                slippage_bps: 50,
                eta_in_seconds: None,
                use_max_amount,
            },
            data: quote_data(destination_address, value),
        },
    );
    input
}

pub(crate) fn mock_utxo_with(transaction_id: &str, vout: i32, value: &str, address: &str) -> UTXO {
    UTXO {
        transaction_id: transaction_id.to_string(),
        vout,
        value: value.to_string(),
        address: address.to_string(),
    }
}

pub(crate) fn mock_utxo_with_address(address: &str) -> UTXO {
    mock_utxo_with(TEST_UTXO_TXID, 0, "50000", address)
}
