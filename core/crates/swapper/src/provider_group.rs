use async_trait::async_trait;
use futures::future::{join_all, try_join_all};
use primitives::{AssetId, Chain};

use crate::{Contracts, FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData, cross_chain::VaultAddresses};

#[derive(Debug)]
pub(crate) struct ProviderGroup {
    provider: ProviderType,
    members: Vec<(&'static str, Box<dyn Swapper>)>,
}

impl ProviderGroup {
    pub fn new(id: SwapperProvider, members: Vec<(&'static str, Box<dyn Swapper>)>) -> Self {
        Self { provider: ProviderType::new(id), members }
    }

    fn members_for(&self, chain: Chain) -> impl Iterator<Item = &(&'static str, Box<dyn Swapper>)> {
        self.members.iter().filter(move |(_, member)| member.supported_chains().contains(&chain))
    }

    fn quote_provider(&self, protocol_id: &str) -> ProviderType {
        ProviderType {
            protocol_id: protocol_id.to_string(),
            ..self.provider.clone()
        }
    }
}

#[async_trait]
impl Swapper for ProviderGroup {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        let assets = self.members.iter().flat_map(|(_, member)| member.supported_assets()).collect::<Vec<_>>();
        assets.iter().enumerate().filter(|(index, asset)| !assets[..*index].contains(asset)).map(|(_, asset)| asset.clone()).collect()
    }

    fn contracts(&self, from_asset: &AssetId) -> Contracts {
        self.members_for(from_asset.chain).map(|(_, member)| member.contracts(from_asset)).collect()
    }

    fn amount_mode(&self, request: &QuoteRequest) -> SwapAmountMode {
        self.members_for(request.from_asset.asset_id().chain).next().map_or(SwapAmountMode::Fixed, |(_, member)| member.amount_mode(request))
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        join_all(self.members_for(from_asset.chain).map(|(_, member)| member.preload_routes(from_asset, to_asset))).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let members = self.members_for(request.from_asset.asset_id().chain);
        let results = join_all(members.map(|(protocol_id, member)| async move { member.get_quote(request).await.map(|quote| (*protocol_id, quote)) })).await;
        let best = results
            .iter()
            .filter_map(|result| result.as_ref().ok())
            .reduce(|best, candidate| if candidate.1.to_value > best.1.to_value { candidate } else { best });
        match best {
            Some((protocol_id, quote)) => Ok(quote.with_provider(self.quote_provider(protocol_id))),
            None => Err(SwapperError::combined(results.into_iter().filter_map(Result::err).collect())),
        }
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let (_, member) = self.members.iter().find(|(protocol_id, _)| *protocol_id == quote.data.provider.protocol_id).ok_or(SwapperError::InvalidRoute)?;
        member.get_quote_data(&quote.with_provider(member.provider().clone()), data).await
    }

    async fn get_vault_addresses(&self, from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        let addresses = try_join_all(self.members.iter().map(|(_, member)| member.get_vault_addresses(from_timestamp))).await?;
        Ok(VaultAddresses {
            deposit: addresses.iter().flat_map(|addresses| addresses.deposit.clone()).collect(),
            send: addresses.iter().flat_map(|addresses| addresses.send.clone()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::Chain;

    use super::*;
    use crate::testkit::MockSwapper;

    fn quote(request: &QuoteRequest, to_value: u64) -> Result<Quote, SwapperError> {
        Ok(Quote {
            to_value: BigUint::from(to_value),
            ..Quote::mock_with_request(request)
        })
    }

    fn group(members: Vec<(&'static str, MockSwapper)>) -> ProviderGroup {
        ProviderGroup::new(SwapperProvider::RaydiumClmm, members.into_iter().map(|(id, member)| (id, Box::new(member) as Box<dyn Swapper>)).collect())
    }

    #[tokio::test]
    async fn test_group_quotes_best_member_and_builds_with_it() {
        let low = MockSwapper::new(SwapperProvider::RaydiumClmm, |request| quote(request, 100));
        let high = MockSwapper::new(SwapperProvider::Orca, |request| quote(request, 200));
        let none = MockSwapper::new(SwapperProvider::PumpSwap, |_| Err(SwapperError::NoQuoteAvailable));
        let (low_builds, high_builds) = (low.builds(), high.builds());
        let group = group(vec![("low", low), ("high", high), ("none", none)]);
        let request = QuoteRequest::mock(Chain::Ethereum, None);

        let quote = group.get_quote(&request).await.unwrap();
        assert_eq!(quote.to_value, BigUint::from(200u64));
        assert_eq!(
            (quote.data.provider.id, quote.data.provider.protocol.as_str(), quote.data.provider.protocol_id.as_str()),
            (SwapperProvider::RaydiumClmm, "Raydium", "high")
        );

        group.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!((low_builds.lock().unwrap().len(), high_builds.lock().unwrap().len()), (0, 1));

        let unknown = quote.with_provider(group.quote_provider("unknown"));
        assert_eq!(group.get_quote_data(&unknown, FetchQuoteData::None).await.err(), Some(SwapperError::InvalidRoute));
    }

    #[tokio::test]
    async fn test_group_asks_only_members_on_the_request_chain() {
        let ethereum = MockSwapper::new(SwapperProvider::UniswapV3, |request| quote(request, 100)).with_supported_assets(vec![SwapperChainAsset::All(Chain::Ethereum)]);
        let solana = MockSwapper::new(SwapperProvider::UniswapV4, |_| Err(SwapperError::NotSupportedChain)).with_supported_assets(vec![SwapperChainAsset::All(Chain::Solana)]);
        let group = ProviderGroup::new(SwapperProvider::UniswapV3, vec![("uniswap_v3", Box::new(ethereum)), ("uniswap_v4", Box::new(solana))]);

        assert_eq!(group.supported_assets(), vec![SwapperChainAsset::All(Chain::Ethereum), SwapperChainAsset::All(Chain::Solana)]);
        assert_eq!(group.get_quote(&QuoteRequest::mock(Chain::Ethereum, None)).await.unwrap().data.provider.protocol_id, "uniswap_v3");
        assert_eq!(group.get_quote(&QuoteRequest::mock(Chain::Solana, None)).await, Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_group_binds_every_member_on_the_chain() {
        let clmm = MockSwapper::new(SwapperProvider::RaydiumClmm, |request| quote(request, 100)).with_contracts(vec!["clmm"]);
        let amm = MockSwapper::new(SwapperProvider::RaydiumClmm, |request| quote(request, 100)).with_contracts(vec!["amm"]);
        let solana = MockSwapper::new(SwapperProvider::RaydiumClmm, |request| quote(request, 100))
            .with_supported_assets(vec![SwapperChainAsset::All(Chain::Solana)])
            .with_contracts(vec!["other_chain"]);
        let group = group(vec![("clmm", clmm), ("amm", amm), ("solana", solana)]);

        assert_eq!(group.contracts(&Chain::Ethereum.as_asset_id()), Contracts::new(vec!["clmm", "amm"]));
    }

    #[tokio::test]
    async fn test_group_reports_errors_like_the_swapper() {
        let request = QuoteRequest::mock(Chain::Ethereum, None);
        let none = || MockSwapper::new(SwapperProvider::Orca, |_| Err(SwapperError::NoQuoteAvailable));
        let minimum = MockSwapper::new(SwapperProvider::Orca, |_| Err(SwapperError::InputAmountError { min_amount: Some("10".into()) }));

        assert_eq!(group(vec![("a", none()), ("b", none())]).get_quote(&request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(group(vec![("a", none()), ("b", minimum)]).get_quote(&request).await, Err(SwapperError::InputAmountError { min_amount: Some("10".into()) }));
    }
}
