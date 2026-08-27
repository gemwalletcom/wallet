use num_bigint::BigInt;
use primitives::swap::{SwapProviderData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId, Chain, Wallet};
use swapper::permit2_data::{Permit2Detail, PermitSingle};
use swapper::{Options, Permit2ApprovalData, Quote, QuoteRequest, SwapperError, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode};

use crate::config::swap_config::{SwapConfig, get_default_slippage};
use crate::services::swap::model::GemSwapTransfer;

pub fn quote_request(wallet: &Wallet, from_asset: &Asset, to_asset: &Asset, value: String, use_max_amount: bool, slippage_bps: Option<u32>) -> Result<QuoteRequest, SwapperError> {
    let wallet_address = account_address(wallet, from_asset.chain())?;
    let destination_address = account_address(wallet, to_asset.chain())?;
    Ok(QuoteRequest {
        from_asset: quote_asset(from_asset),
        to_asset: quote_asset(to_asset),
        wallet_address,
        destination_address,
        value,
        options: Options {
            slippage: slippage(from_asset, slippage_bps),
            use_max_amount,
        },
    })
}

pub fn sort_quotes(mut quotes: Vec<Quote>) -> Vec<Quote> {
    quotes.sort_by_key(|quote| std::cmp::Reverse(to_value(quote)));
    quotes
}

pub fn swap_transfer(wallet: &Wallet, quote: &Quote, data: SwapQuoteData) -> Result<GemSwapTransfer, SwapperError> {
    let to_chain = AssetId::new(&quote.request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?.chain;
    Ok(GemSwapTransfer {
        quote: swap_quote(quote),
        data,
        recipient: account_address(wallet, to_chain)?,
        value: quote.request.value.clone(),
        use_max_amount: quote.request.options.use_max_amount,
    })
}

pub fn swap_quote(quote: &Quote) -> SwapQuote {
    SwapQuote {
        from_address: quote.request.wallet_address.clone(),
        from_value: quote.from_value.clone(),
        min_from_value: quote.min_from_value.clone(),
        to_address: quote.request.destination_address.clone(),
        to_value: quote.to_value.clone(),
        provider_data: SwapProviderData {
            provider: quote.data.provider.id,
            name: quote.data.provider.name.clone(),
            protocol_name: quote.data.provider.protocol.clone(),
        },
        slippage_bps: quote.data.slippage_bps,
        eta_in_seconds: quote.eta_in_seconds,
        use_max_amount: Some(quote.request.options.use_max_amount),
    }
}

pub fn permit_single(approval: &Permit2ApprovalData, now: u64, config: &SwapConfig) -> PermitSingle {
    PermitSingle {
        details: Permit2Detail {
            token: approval.token.clone(),
            amount: approval.value.clone(),
            expiration: now + config.permit2_expiration,
            nonce: approval.permit2_nonce,
        },
        spender: approval.spender.clone(),
        sig_deadline: now + config.permit2_sig_deadline,
    }
}

fn slippage(from_asset: &Asset, slippage_bps: Option<u32>) -> SwapperSlippage {
    match slippage_bps {
        Some(bps) => SwapperSlippage {
            bps,
            mode: SwapperSlippageMode::Exact,
        },
        None => SwapperSlippage {
            mode: SwapperSlippageMode::Auto,
            ..get_default_slippage(&from_asset.chain())
        },
    }
}

fn quote_asset(asset: &Asset) -> SwapperQuoteAsset {
    SwapperQuoteAsset {
        id: asset.id.to_string(),
        symbol: asset.symbol.clone(),
        decimals: asset.decimals as u32,
    }
}

fn account_address(wallet: &Wallet, chain: Chain) -> Result<String, SwapperError> {
    wallet
        .accounts
        .iter()
        .find(|account| account.chain == chain)
        .map(|account| account.address.clone())
        .ok_or(SwapperError::NotSupportedChain)
}

fn to_value(quote: &Quote) -> BigInt {
    quote.to_value.parse().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, AssetId, AssetType, Chain, WalletId, WalletSource, WalletType};

    fn wallet(chains: &[Chain]) -> Wallet {
        Wallet {
            id: WalletId::Multicoin("0x1".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type: WalletType::Multicoin,
            accounts: chains
                .iter()
                .map(|chain| Account {
                    chain: *chain,
                    address: format!("{chain}-address"),
                    derivation_path: String::new(),
                    extended_public_key: None,
                })
                .collect(),
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        }
    }

    fn asset(chain: Chain) -> Asset {
        Asset::new(AssetId::from_chain(chain), chain.to_string(), chain.to_string().to_uppercase(), 18, AssetType::NATIVE)
    }

    #[test]
    fn test_quote_request_uses_wallet_accounts_and_slippage() {
        let wallet = wallet(&[Chain::Ethereum, Chain::Solana]);
        let request = quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), true, Some(50)).unwrap();
        assert_eq!(request.wallet_address, "ethereum-address");
        assert_eq!(request.destination_address, "solana-address");
        assert_eq!(
            request.options.slippage,
            SwapperSlippage {
                bps: 50,
                mode: SwapperSlippageMode::Exact
            }
        );
        assert!(request.options.use_max_amount);

        let auto = quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), false, None).unwrap();
        assert_eq!(auto.options.slippage.mode, SwapperSlippageMode::Auto);
        assert_eq!(auto.options.slippage.bps, get_default_slippage(&Chain::Ethereum).bps);
    }

    #[test]
    fn test_quote_request_requires_accounts() {
        let wallet = wallet(&[Chain::Ethereum]);
        assert!(matches!(
            quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "1".to_string(), false, None),
            Err(SwapperError::NotSupportedChain)
        ));
    }

    #[test]
    fn test_swap_transfer_maps_quote_and_recipient() {
        let wallet = wallet(&[Chain::Ethereum, Chain::Solana]);
        let request = quote_request(&wallet, &asset(Chain::Ethereum), &asset(Chain::Solana), "100".to_string(), true, Some(50)).unwrap();
        let quote = Quote {
            from_value: "99".to_string(),
            min_from_value: Some("90".to_string()),
            to_value: "1".to_string(),
            data: swapper::ProviderData {
                provider: swapper::ProviderType::new(swapper::SwapperProvider::Jupiter),
                slippage_bps: 50,
                routes: vec![],
            },
            request,
            eta_in_seconds: Some(30),
        };
        let data = SwapQuoteData {
            to: "0xrouter".to_string(),
            data_type: primitives::swap::SwapQuoteDataType::Contract,
            value: "100".to_string(),
            data: "0x".to_string(),
            memo: None,
            approval: None,
            gas_limit: None,
        };

        let transfer = swap_transfer(&wallet, &quote, data.clone()).unwrap();

        assert_eq!(transfer.recipient, "solana-address");
        assert_eq!(transfer.value, "100");
        assert!(transfer.use_max_amount);
        assert_eq!(transfer.data, data);
        assert_eq!(transfer.quote.from_address, "ethereum-address");
        assert_eq!(transfer.quote.to_address, "solana-address");
        assert_eq!(transfer.quote.from_value, "99");
        assert_eq!(transfer.quote.min_from_value.as_deref(), Some("90"));
        assert_eq!(transfer.quote.provider_data.provider, swapper::SwapperProvider::Jupiter);
        assert_eq!(transfer.quote.slippage_bps, 50);
        assert_eq!(transfer.quote.use_max_amount, Some(true));

        let ethereum_only = super::tests::wallet(&[Chain::Ethereum]);
        assert!(matches!(swap_transfer(&ethereum_only, &quote, data), Err(SwapperError::NotSupportedChain)));
    }

    #[test]
    fn test_permit_single_deadlines() {
        let approval = Permit2ApprovalData {
            token: "0xtoken".to_string(),
            spender: "0xspender".to_string(),
            value: "1".to_string(),
            permit2_contract: "0xpermit2".to_string(),
            permit2_nonce: 7,
        };
        let config = crate::config::swap_config::get_swap_config();
        let permit = permit_single(&approval, 1_000, &config);
        assert_eq!(permit.details.expiration, 1_000 + config.permit2_expiration);
        assert_eq!(permit.sig_deadline, 1_000 + config.permit2_sig_deadline);
        assert_eq!(permit.details.nonce, 7);
        assert_eq!(permit.spender, "0xspender");
    }
}
