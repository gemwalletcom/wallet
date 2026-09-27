use crate::{
    AlienError, AssetList, FetchQuoteData, Permit2ApprovalData, ProviderData, ProviderType, Route, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData,
    SwapperSlippage, SwapperSlippageMode, Target,
};
use async_trait::async_trait;
use gem_jsonrpc::RpcResponse;
use gem_jsonrpc::rpc::RpcProvider as GenericRpcProvider;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_TOKEN_ID};
use std::sync::{Arc, Mutex};

use super::{Options, Quote, QuoteRequest};

impl ProviderData {
    pub fn mock() -> Self {
        ProviderData {
            provider: ProviderType::new(SwapperProvider::Okx),
            routes: vec![],
            slippage_bps: 50,
        }
    }
}

impl AssetList {
    pub fn mock_with_chains(chains: &[Chain]) -> Self {
        Self {
            chains: chains.to_vec(),
            asset_ids: Vec::new(),
        }
    }

    pub fn mock() -> Self {
        Self {
            chains: vec![Chain::Tron, Chain::Bitcoin],
            asset_ids: vec![
                AssetId::from_chain(Chain::Bitcoin),
                AssetId::from_token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
                AssetId::from_chain(Chain::Ethereum),
            ],
        }
    }
}

impl Route {
    pub fn mock(input: AssetId, output: AssetId) -> Self {
        Route {
            input,
            output,
            route_data: serde_json::json!({
                "fee_tier": "100",
                "min_amount_out": "1",
            })
            .to_string(),
        }
    }
}

impl Options {
    pub fn mock_exact(bps: u32) -> Self {
        Self::new_with_slippage(SwapperSlippage::mock_exact(bps))
    }
}

impl QuoteRequest {
    pub fn mock(chain: Chain, token_id: Option<&str>) -> Self {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from(chain, token_id.map(|s| s.to_string()))),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(chain)),
            wallet_address: "address".to_string(),
            destination_address: "address".to_string(),
            value: BigUint::from(1000000u64),
            options: Options::default(),
        }
    }
}

impl Quote {
    pub fn mock(chain: Chain, token_id: Option<&str>) -> Self {
        Quote {
            from_value: BigUint::from(1000000u64),
            min_from_value: None,
            to_value: BigUint::from(1000000u64),
            data: ProviderData::mock(),
            request: QuoteRequest::mock(chain, token_id),
            eta_in_seconds: None,
        }
    }

    pub fn mock_with_request(request: &QuoteRequest) -> Self {
        Quote {
            from_value: request.value.clone(),
            request: request.clone(),
            ..Self::mock_with_provider(SwapperProvider::UniswapV3, "1")
        }
    }

    pub fn mock_with_provider(provider: SwapperProvider, to_value: &str) -> Self {
        Quote {
            from_value: BigUint::from(1000000u64),
            min_from_value: None,
            to_value: to_value.parse().unwrap(),
            data: ProviderData {
                provider: ProviderType::new(provider),
                routes: vec![],
                slippage_bps: 50,
            },
            request: QuoteRequest::mock(Chain::Ethereum, None),
            eta_in_seconds: None,
        }
    }
}

pub fn mock_quote(from_asset: SwapperQuoteAsset, to_asset: SwapperQuoteAsset) -> QuoteRequest {
    QuoteRequest {
        from_asset,
        to_asset,
        wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        value: BigUint::from(1000000u64),
        options: Options {
            slippage: SwapperSlippage { mode: SwapperSlippageMode::Auto, bps: 50 },
            use_max_amount: false,
        },
    }
}

pub fn mock_bitcoin_max_quote(to_asset: SwapperQuoteAsset) -> QuoteRequest {
    let mut request = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)), to_asset);
    request.wallet_address = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".into();
    request.destination_address = "11111111111111111111111111111111".into();
    request.value = BigUint::from(89100u64);
    request.options.use_max_amount = true;
    request
}

pub fn mock_ton(wallet_address: String) -> QuoteRequest {
    QuoteRequest {
        from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)),
        to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)),
        wallet_address: wallet_address.clone(),
        destination_address: wallet_address,
        value: BigUint::from(1000000000u64),
        options: Options { slippage: 100.into(), use_max_amount: false },
    }
}

impl crate::swapper::GemSwapper {
    pub fn mock(swappers: Vec<Box<dyn Swapper>>) -> Self {
        Self {
            rpc_provider: Arc::new(UnusedRpcProvider),
            swappers,
        }
    }
}

#[derive(Debug)]
struct UnusedRpcProvider;

#[async_trait]
impl GenericRpcProvider for UnusedRpcProvider {
    type Error = AlienError;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        panic!("a mock swapper never reaches the network: {target:?}")
    }
}

impl RpcProvider for UnusedRpcProvider {
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        panic!("a mock swapper never asks for a node: {chain}")
    }
}

type MockResponse = fn(&QuoteRequest) -> Result<Quote, SwapperError>;

#[derive(Debug)]
pub struct MockSwapper {
    provider: ProviderType,
    supported_assets: Vec<SwapperChainAsset>,
    response: MockResponse,
    amount_mode: SwapAmountMode,
    pending_permit: Option<Permit2ApprovalData>,
    builds: Arc<Mutex<Vec<FetchQuoteData>>>,
}

impl MockSwapper {
    pub fn new(provider: SwapperProvider, response: MockResponse) -> Self {
        Self {
            provider: ProviderType::new(provider),
            supported_assets: vec![SwapperChainAsset::All(Chain::Ethereum)],
            response,
            amount_mode: SwapAmountMode::Fixed,
            pending_permit: None,
            builds: Arc::default(),
        }
    }

    pub fn with_amount_mode(self, amount_mode: SwapAmountMode) -> Self {
        Self { amount_mode, ..self }
    }

    pub fn with_pending_permit(self, permit: Permit2ApprovalData) -> Self {
        Self { pending_permit: Some(permit), ..self }
    }

    pub fn builds(&self) -> Arc<Mutex<Vec<FetchQuoteData>>> {
        self.builds.clone()
    }
}

#[async_trait]
impl Swapper for MockSwapper {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        self.supported_assets.clone()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        self.amount_mode
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let mut quote = (self.response)(request)?;
        quote.data.provider = self.provider.clone();
        Ok(quote)
    }

    async fn get_quote_data(&self, _quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let permit2 = match data {
            FetchQuoteData::Permit2(_) => None,
            FetchQuoteData::EstimateGas | FetchQuoteData::None => self.pending_permit.clone(),
        };
        self.builds.lock().unwrap().push(data);
        Ok(SwapperQuoteData { permit2, ..SwapperQuoteData::mock() })
    }
}
