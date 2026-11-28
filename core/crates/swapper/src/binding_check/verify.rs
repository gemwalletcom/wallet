use primitives::{AssetId, swap::SwapQuoteDataType};

use super::{
    amounts::verify_amounts,
    data_format::DataFormat,
    evm::verify_evm_call,
    solana::{StepLimits, verify_solana},
    ton::verify_ton_messages,
};
use crate::{Binding, Contracts, Deployment, Quote, SwapperError, SwapperQuoteData};

pub(crate) fn verify(quote: &Quote, binding: &Binding, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let from_asset = quote.request.from_asset.asset_id();
    verify_amounts(&from_asset, &quote.from_value, data)?;
    match binding {
        Binding::Contracts(contracts) => verify_contracts(quote, &from_asset, contracts, data),
        Binding::DepositAddress | Binding::BuiltOnDevice => Ok(()),
    }
}

fn verify_contracts(quote: &Quote, from_asset: &AssetId, contracts: &Contracts, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    if contracts.targets.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    match (DataFormat::of(from_asset.chain.chain_type()), &data.data_type) {
        (DataFormat::SolanaTransaction, SwapQuoteDataType::Transfer | SwapQuoteDataType::Contract) => verify_solana(&data.data, &quote.request.wallet_address, &contracts.targets, &StepLimits::from_quote(quote)?),
        (DataFormat::TonMessages, SwapQuoteDataType::Transfer | SwapQuoteDataType::Contract) => verify_ton_messages(&data.data, &contracts.targets, &quote.from_value),
        (DataFormat::EvmCall | DataFormat::Unread, SwapQuoteDataType::Transfer) => match Deployment::contains(from_asset.chain, &contracts.targets, &data.to) {
            true => Ok(()),
            false => Err(SwapperError::InvalidRoute),
        },
        (DataFormat::EvmCall, SwapQuoteDataType::Contract) => verify_evm_call(from_asset, contracts, data),
        (DataFormat::Unread, SwapQuoteDataType::Contract) => Err(SwapperError::InvalidRoute),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QuoteRequest, solana_dex::SolanaTransaction};
    use gem_solana::{JUPITER_PROGRAM_ID, compute_budget::set_compute_unit_limit};
    use num_bigint::BigUint;
    use primitives::{
        Chain,
        testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_SOLANA_SENDER},
    };

    #[test]
    fn test_verify() {
        let quote = Quote::mock(Chain::Ethereum, None);
        let data = SwapperQuoteData::mock();

        assert_eq!(verify(&quote, &Binding::BuiltOnDevice, &data), Ok(()));
        assert_eq!(verify(&quote, &Binding::DepositAddress, &data), Ok(()));
        assert_eq!(verify(&quote, &Binding::Contracts(Contracts::new(vec![TEST_EVM_RECIPIENT])), &data), Ok(()));
        assert_eq!(
            verify(&quote, &Binding::Contracts(Contracts::default()), &data),
            Err(SwapperError::InvalidRoute),
            "a provider that ships no contracts binds nothing"
        );
        let solana = Quote {
            request: QuoteRequest {
                wallet_address: TEST_SOLANA_SENDER.to_string(),
                ..QuoteRequest::mock(Chain::Solana, None)
            },
            ..Quote::mock(Chain::Solana, None)
        };
        let standard_steps = SolanaTransaction::new(vec![set_compute_unit_limit(200_000)], vec![]).encode(TEST_SOLANA_SENDER, &[]).unwrap();
        assert_eq!(verify(&solana, &Binding::Contracts(Contracts::new(vec![JUPITER_PROGRAM_ID])), &standard_steps), Ok(()));
        assert_eq!(
            verify(&quote, &Binding::BuiltOnDevice, &SwapperQuoteData { value: BigUint::from(1_000_001u64), ..data }),
            Err(SwapperError::InvalidRoute),
            "the amounts bind every provider, bound or not"
        );

        let xrp = Quote::mock(Chain::Xrp, None);
        let deposit = SwapperQuoteData::new_transfer("rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn".to_string(), xrp.from_value.clone(), None);
        assert_eq!(verify(&xrp, &Binding::Contracts(Contracts::new(vec!["rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn"])), &deposit), Ok(()));
        assert_eq!(
            verify(&xrp, &Binding::Contracts(Contracts::new(vec!["rDsbeomae4FXwgQTJp9Rs64Qg9vDiTCdBv"])), &deposit),
            Err(SwapperError::InvalidRoute),
            "a transfer pays only a shipped address"
        );
        assert_eq!(
            verify(&Quote::mock(Chain::Sui, None), &Binding::Contracts(Contracts::new(vec![TEST_EVM_RECIPIENT])), &SwapperQuoteData::mock()),
            Err(SwapperError::InvalidRoute),
            "a contract call on a network the check cannot read is refused"
        );
    }
}
