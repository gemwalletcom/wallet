use async_trait::async_trait;
use borsh::BorshDeserialize;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use gem_solana::{
    DEFAULT_SWAP_GAS_LIMIT, Pubkey, SolanaRpc, SolanaRpcConfig,
    instructions::program_ids::{sysvar_clock, sysvar_program},
    models::{AccountData, Clock, ValueResult, blockhash::SolanaBlockhashResult},
};
use num_bigint::BigUint;
use primitives::Chain;
use serde_json::Value;
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt,
    sync::Arc,
};

use super::{
    protocol::{DexProtocol, PoolSwap},
    route::{DexHop, DexRoute, preferred_fee_side},
    token::{INTERMEDIATE_TOKENS, SwapToken, decode_mint},
    transaction::DexTransaction,
};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData,
    client_factory::create_client_with_chain,
    error::INVALID_ADDRESS,
    fees::{FeeSide, default_referral_fees},
    route_cache::{Cache, DiscoveryCache, OutpricedCache},
    route_path::RoutePath,
};

type AccountsResult = ValueResult<Vec<Option<AccountData>>>;
type Accounts = HashMap<Pubkey, Option<AccountData>>;
type Pair = (Pubkey, Pubkey);
type PoolRoute<Pool> = Vec<(DexHop, Pool)>;

struct PoolLookup<Pool> {
    pair: Pair,
    known: Vec<(Pubkey, Pool)>,
    addresses: Vec<Pubkey>,
}

impl<Pool: Clone> PoolLookup<Pool> {
    fn without(&self, skipped: &HashSet<Pubkey>) -> Self {
        Self {
            pair: self.pair,
            known: self.known.iter().filter(|(address, _)| !skipped.contains(address)).cloned().collect(),
            addresses: self.addresses.iter().filter(|address| !skipped.contains(address)).copied().collect(),
        }
    }
}

type PathLookup<Pool> = (RoutePath<Pubkey>, Vec<PoolLookup<Pool>>);

const SIZE_CHECK_BLOCKHASH: &str = "11111111111111111111111111111111";

pub(crate) struct SolanaDex<P: DexProtocol, C: Client + Clone = RpcClient> {
    provider: ProviderType,
    protocol: P,
    client: JsonRpcClient<C>,
    pools: DiscoveryCache<(Pubkey, P::Pool), Pubkey, Pubkey>,
    constant_accounts: Cache<Pubkey, Option<AccountData>>,
    outpriced_routes: OutpricedCache<Vec<Pubkey>, Pubkey>,
}

impl<P: DexProtocol> SolanaDex<P> {
    pub fn new(protocol: P, rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        Some(Self::with_client(protocol, create_client_with_chain(rpc_provider, Chain::Solana).ok()?))
    }
}

impl<P: DexProtocol, C: Client + Clone + Send + Sync + 'static> SolanaDex<P, C> {
    pub fn with_client(protocol: P, client: JsonRpcClient<C>) -> Self {
        Self {
            provider: ProviderType::new(P::PROVIDER),
            protocol,
            client,
            pools: DiscoveryCache::default(),
            constant_accounts: Cache::default(),
            outpriced_routes: OutpricedCache::default(),
        }
    }

    fn path_lookups(&self, (input_mint, output_mint): Pair) -> Result<Vec<PathLookup<P::Pool>>, SwapperError> {
        let paths = RoutePath::all(&input_mint, &output_mint, INTERMEDIATE_TOKENS.iter().map(|token| token.mint), |left, right| left == right);
        let lookups = paths
            .into_iter()
            .map(|path| Ok((path.clone(), path.pairs(&input_mint, &output_mint).into_iter().map(|pair| self.pool_lookup(pair)).collect::<Result<Vec<_>, SwapperError>>()?)))
            .collect::<Result<Vec<_>, SwapperError>>()?;
        Ok(lookups.into_iter().filter(|(_, lookups)| has_pools(lookups)).collect())
    }

    fn live_paths(&self, paths: &[PathLookup<P::Pool>], pair: Pair, value: u64) -> (Vec<PathLookup<P::Pool>>, bool) {
        let skipped = paths.iter().flat_map(|(path, lookups)| self.outpriced_pools(path, lookups, pair, value)).collect::<HashSet<_>>();
        let live = paths
            .iter()
            .map(|(path, lookups)| (path.clone(), lookups.iter().map(|lookup| lookup.without(&skipped)).collect::<Vec<_>>()))
            .filter(|(_, lookups)| has_pools(lookups))
            .collect();
        (live, !skipped.is_empty())
    }

    fn outpriced_pools(&self, path: &RoutePath<Pubkey>, lookups: &[PoolLookup<P::Pool>], pair: Pair, value: u64) -> Vec<Pubkey> {
        let known = lookups.iter().map(|lookup| (lookup.pair, lookup.known.clone())).collect();
        let (live, outpriced) = self.outpriced_routes.partition(&pair.0, &pair.1, u128::from(value), self.routes([path], pair, &known), |route| route_pools(route));
        if live.is_empty() && !outpriced.is_empty() {
            lookups.iter().flat_map(|lookup| &lookup.known).map(|(address, _)| *address).collect()
        } else {
            outpriced.iter().flat_map(|route| route_pools(route)).collect()
        }
    }

    fn requested_accounts(&self, paths: &[PathLookup<P::Pool>], pair: Pair) -> Vec<Pubkey> {
        let lookups = paths.iter().flat_map(|(_, lookups)| lookups).collect::<Vec<_>>();
        let known = lookups.iter().map(|lookup| (lookup.pair, lookup.known.clone())).collect();
        let predicted = self.predicted_quote_accounts(&self.routes(paths.iter().map(|(path, _)| path), pair, &known));
        lookups.iter().flat_map(|lookup| lookup.addresses.clone()).chain(predicted).collect()
    }

    fn pool_lookup(&self, pair: Pair) -> Result<PoolLookup<P::Pool>, SwapperError> {
        let (input_mint, output_mint) = pair;
        let probes = self.protocol.pool_addresses(&input_mint, &output_mint)?;
        let known = self.pools.candidates_for_probes(input_mint, output_mint, &probes);
        let addresses = known.iter().map(|(address, _)| *address).chain(self.pools.missing_probes(input_mint, output_mint, &probes)).collect();
        Ok(PoolLookup { pair, known, addresses })
    }

    fn record_pools(&self, lookup: &PoolLookup<P::Pool>, accounts: &Accounts, timestamp: u64) -> Vec<(Pubkey, P::Pool)> {
        let (input_mint, output_mint) = lookup.pair;
        let (usable, unusable): (Vec<_>, Vec<_>) = lookup
            .addresses
            .iter()
            .map(|address| (*address, self.decode_pool(accounts, address)))
            .partition(|(_, pool)| pool.as_ref().is_none_or(|(_, pool)| self.protocol.is_tradable(pool, timestamp)));
        let pools = usable.iter().filter_map(|(_, pool)| pool.clone()).collect();
        self.pools.record_discovery(input_mint, output_mint, usable);
        self.pools.record_unusable(input_mint, output_mint, unusable.into_iter().map(|(address, _)| address));
        pools
    }

    fn constant_route_accounts<'a>(&self, pools: impl IntoIterator<Item = &'a P::Pool>) -> HashSet<Pubkey>
    where
        P::Pool: 'a,
    {
        pools.into_iter().flat_map(|pool| self.protocol.constant_accounts(pool)).collect()
    }

    fn decode_pool(&self, accounts: &Accounts, address: &Pubkey) -> Option<(Pubkey, P::Pool)> {
        Some((*address, self.protocol.decode_pool(&Self::program_data(accounts, address).ok()??).ok()?))
    }

    fn routes<'a>(&self, paths: impl IntoIterator<Item = &'a RoutePath<Pubkey>>, (input_mint, output_mint): Pair, pools: &HashMap<Pair, Vec<(Pubkey, P::Pool)>>) -> Vec<PoolRoute<P::Pool>> {
        let hops = |(input_mint, output_mint): Pair| {
            pools
                .get(&(input_mint, output_mint))
                .into_iter()
                .flatten()
                .map(move |(address, pool)| (DexHop { pool: *address, input_mint, output_mint }, pool.clone()))
        };
        paths
            .into_iter()
            .flat_map(|path| match path {
                RoutePath::Direct => hops((input_mint, output_mint)).map(|hop| vec![hop]).collect(),
                RoutePath::Intermediary(_) => path
                    .pairs(&input_mint, &output_mint)
                    .into_iter()
                    .map(|pair| hops(pair).max_by_key(|(_, pool)| self.protocol.liquidity(pool)))
                    .collect::<Option<Vec<_>>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
            })
            .collect()
    }

    fn route_quote_accounts(&self, route: &[(DexHop, P::Pool)]) -> Option<Vec<Vec<Pubkey>>> {
        route.iter().map(|(hop, pool)| self.protocol.quote_accounts(&hop.pool, pool, &hop.input_mint).ok()).collect()
    }

    fn predicted_quote_accounts(&self, routes: &[PoolRoute<P::Pool>]) -> Vec<Pubkey> {
        routes.iter().filter_map(|route| self.route_quote_accounts(route)).flatten().flatten().collect()
    }

    async fn quote_routes(&self, routes: Vec<PoolRoute<P::Pool>>, amount_in: u64, accounts: Accounts, timestamp: u64) -> Result<Vec<(PoolRoute<P::Pool>, Option<u64>)>, SwapperError> {
        let route_accounts = routes.iter().map(|route| self.route_quote_accounts(route)).collect::<Vec<_>>();
        let missing = route_accounts.iter().flatten().flatten().flatten().filter(|address| !accounts.contains_key(address)).copied().collect();
        let constants = self.constant_route_accounts(routes.iter().flatten().map(|(_, pool)| pool));
        let accounts = accounts.into_iter().chain(self.load_accounts(missing, &constants).await?).collect::<Accounts>();
        Ok(routes
            .into_iter()
            .zip(route_accounts)
            .map(|(route, hop_accounts)| {
                let output_value = hop_accounts.and_then(|hop_accounts| {
                    route.iter().zip(hop_accounts).try_fold(amount_in, |amount, ((hop, pool), addresses)| {
                        let data = addresses.iter().map(|account| Self::program_data(&accounts, account)).collect::<Result<Vec<_>, _>>().ok()?;
                        self.protocol.quote(&hop.pool, pool, &hop.input_mint, amount, &data, timestamp).ok()
                    })
                });
                (route, output_value)
            })
            .collect())
    }

    async fn quote_paths(&self, request: &QuoteRequest, input: &SwapToken, output: &SwapToken, paths: Vec<PathLookup<P::Pool>>) -> Result<DexRoute, SwapperError> {
        let value = request.value_u64()?;
        let referral_fee = default_referral_fees().solana;
        let referral = Pubkey::from_base58(&referral_fee.address)?;
        let preferred_side = preferred_fee_side(&input.mint, &output.mint);
        let referral_accounts = match preferred_side {
            Some(_) => vec![],
            None => vec![input.account(&referral)?, output.account(&referral)?],
        };
        let pair = (input.mint, output.mint);
        let constant_extras = [referral_accounts, vec![input.mint, output.mint]].concat();
        let requested = [self.requested_accounts(&paths, pair), constant_extras.clone(), vec![sysvar_clock()]].concat();
        let known_pools = paths.iter().flat_map(|(_, lookups)| lookups).flat_map(|lookup| &lookup.known).map(|(_, pool)| pool);
        let constants = constant_extras.iter().copied().chain(self.constant_route_accounts(known_pools)).collect();
        let accounts = self.load_accounts(requested, &constants).await?;
        let clock = chain_clock(&accounts)?;
        let timestamp = u64::try_from(clock.unix_timestamp).unwrap_or_default();
        let pools = paths.iter().flat_map(|(_, lookups)| lookups).map(|lookup| (lookup.pair, self.record_pools(lookup, &accounts, timestamp))).collect();
        let routes = self.routes(paths.iter().map(|(path, _)| path), pair, &pools);
        let mint = |token: &SwapToken| decode_mint(owned_data(&accounts, &token.mint, &token.token_program)?);
        let (input_mint, output_mint) = (mint(input)?, mint(output)?);
        let fee_side = match preferred_side {
            Some(side) => side,
            None => referral_fee_side(&accounts, &referral, input, output)?,
        };
        let swap_value = referral_fee.swap_value(fee_side, value);
        let pool_input = swap_value.saturating_sub(input_mint.transfer_fee(clock.epoch, swap_value));
        let authority = Pubkey::from_base58(&request.wallet_address).map_err(|_| SwapperError::transaction_error(INVALID_ADDRESS))?;
        let quotes = self.quote_routes(routes, pool_input, accounts, timestamp).await?;
        self.outpriced_routes
            .record(&input.mint, &output.mint, u128::from(value), quotes.iter().map(|(route, pool_output)| (route_pools(route), pool_output.map_or(0, u128::from))));
        ranked(quotes)
            .into_iter()
            .map(|(pools, pool_output)| {
                let output_value = pool_output.saturating_sub(output_mint.transfer_fee(clock.epoch, pool_output));
                let route = DexRoute {
                    hops: pools.iter().map(|(hop, _)| *hop).collect(),
                    swap_value,
                    output_value,
                    fee: referral_fee.transfer(fee_side, value, output_value),
                };
                (pools.into_iter().map(|(hop, pool)| (hop.pool, pool)).collect::<Vec<_>>(), route)
            })
            .find(|(pools, route)| {
                route.received_value() > 0
                    && self
                        .transaction(route, pools, input, output, authority, request.options.slippage.bps)
                        .and_then(|transaction| transaction.encode(SIZE_CHECK_BLOCKHASH))
                        .is_ok()
            })
            .map(|(_, route)| route)
            .ok_or(SwapperError::NoQuoteAvailable)
    }

    fn transaction<'a>(&self, route: &DexRoute, pools: &[(Pubkey, P::Pool)], input: &'a SwapToken, output: &'a SwapToken, authority: Pubkey, slippage_bps: u32) -> Result<DexTransaction<'a>, SwapperError> {
        let swap = PoolSwap {
            authority,
            input,
            intermediate: route.intermediate(&input.mint, &output.mint)?,
            output,
            amount: route.swap_value,
            minimum_out: route.minimum_output(slippage_bps),
        };
        let swap_instruction = match (pools, swap.intermediate) {
            ([(address, pool)], None) => self.protocol.swap_instruction(address, pool, &swap),
            ([first, second], Some(intermediate)) => self.protocol.two_hop_swap_instruction(first, second, &swap, intermediate),
            _ => Err(SwapperError::InvalidRoute),
        }?;
        Ok(DexTransaction {
            swap_instruction,
            swap,
            referral: Pubkey::from_base58(&default_referral_fees().solana.address)?,
            fee: route.fee,
            intermediate_token_account: P::INTERMEDIATE_TOKEN_ACCOUNT,
        })
    }

    async fn load_accounts(&self, addresses: Vec<Pubkey>, constants: &HashSet<Pubkey>) -> Result<Accounts, SwapperError> {
        let (cached, missing): (Vec<_>, Vec<_>) = addresses.into_iter().map(|address| (address, self.constant_accounts.get(&address))).partition(|(_, account)| account.is_some());
        let fetched = self.fetch_accounts(missing.into_iter().map(|(address, _)| address).collect()).await?;
        fetched
            .iter()
            .filter(|(address, _)| constants.contains(address))
            .for_each(|(address, account)| self.constant_accounts.put(*address, account.clone()));
        Ok(cached.into_iter().filter_map(|(address, account)| Some((address, account?))).chain(fetched).collect())
    }

    async fn fetch_accounts(&self, addresses: Vec<Pubkey>) -> Result<Accounts, SwapperError> {
        let addresses = addresses.into_iter().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        if addresses.is_empty() {
            return Ok(Accounts::new());
        }
        let response: AccountsResult = self.client.request(accounts_request(&addresses)).await?;
        if response.value.len() != addresses.len() {
            return Err(SwapperError::compute_quote_error("Unexpected Solana accounts response"));
        }
        Ok(addresses.into_iter().zip(response.value).collect())
    }

    async fn fetch_pools_and_blockhash(&self, addresses: Vec<Pubkey>) -> Result<(Vec<(Pubkey, P::Pool)>, String), SwapperError> {
        let requests = vec![accounts_request(&addresses), SolanaRpc::GetLatestBlockhash(SolanaRpcConfig::Confirmed)];
        let [accounts, blockhash]: [Value; 2] = self.client.requests(requests).await?.take_all()?.try_into().map_err(|_| SwapperError::transaction_error("Unexpected Solana batch response"))?;
        let accounts: Accounts = addresses.iter().copied().zip(serde_json::from_value::<AccountsResult>(accounts)?.value).collect();
        let pools = addresses
            .into_iter()
            .map(|address| {
                let data = Self::program_data(&accounts, &address)?.ok_or(SwapperError::InvalidRoute)?;
                Ok((address, self.protocol.decode_pool(&data)?))
            })
            .collect::<Result<Vec<_>, SwapperError>>()?;
        let blockhash: SolanaBlockhashResult = serde_json::from_value(blockhash)?;
        Ok((pools, blockhash.value.blockhash))
    }

    fn program_data(accounts: &Accounts, address: &Pubkey) -> Result<Option<Vec<u8>>, SwapperError> {
        owned_data(accounts, address, &P::PROGRAM_ID)
    }
}

impl<P: DexProtocol, C: Client + Clone> fmt::Debug for SolanaDex<P, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolanaDex").field("provider", &self.provider.id).finish()
    }
}

fn has_pools<Pool>(lookups: &[PoolLookup<Pool>]) -> bool {
    lookups.iter().all(|lookup| !lookup.addresses.is_empty())
}

fn route_pools<Pool>(route: &[(DexHop, Pool)]) -> Vec<Pubkey> {
    route.iter().map(|(hop, _)| hop.pool).collect()
}

fn ranked<Pool>(quotes: Vec<(PoolRoute<Pool>, Option<u64>)>) -> Vec<(PoolRoute<Pool>, u64)> {
    let ranked = quotes
        .into_iter()
        .enumerate()
        .filter_map(|(index, (route, value))| Some(((Reverse(value?), route.len(), index), (route, value?))))
        .collect::<BTreeMap<_, _>>();
    ranked.into_values().collect()
}

fn owned_account<'a>(accounts: &'a Accounts, address: &Pubkey, owner: &Pubkey) -> Option<&'a AccountData> {
    accounts.get(address).and_then(Option::as_ref).filter(|account| account.owner == owner.to_base58())
}

fn owned_data(accounts: &Accounts, address: &Pubkey, owner: &Pubkey) -> Result<Option<Vec<u8>>, SwapperError> {
    Ok(owned_account(accounts, address, owner).map(AccountData::decoded_data).transpose()?)
}

fn chain_clock(accounts: &Accounts) -> Result<Clock, SwapperError> {
    let data = owned_data(accounts, &sysvar_clock(), &sysvar_program())?.ok_or_else(|| SwapperError::compute_quote_error("Missing Solana clock"))?;
    Clock::try_from_slice(&data).map_err(SwapperError::compute_quote_error)
}

fn accounts_request(addresses: &[Pubkey]) -> SolanaRpc {
    SolanaRpc::GetMultipleAccounts(addresses.iter().map(Pubkey::to_base58).collect())
}

fn referral_fee_side(accounts: &Accounts, referral: &Pubkey, input: &SwapToken, output: &SwapToken) -> Result<FeeSide, SwapperError> {
    let exists = |token: &SwapToken| Ok::<_, SwapperError>(owned_account(accounts, &token.account(referral)?, &token.token_program).is_some());
    FeeSide::from_available(exists(input)?, exists(output)?).ok_or_else(|| SwapperError::compute_quote_error("Referral fee account is unavailable"))
}

#[async_trait]
impl<P: DexProtocol, C: Client + Clone + Send + Sync + 'static> Swapper for SolanaDex<P, C> {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Solana)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let input = SwapToken::from_asset(&request.from_asset)?;
        let output = SwapToken::from_asset(&request.to_asset)?;
        let value = request.value_u64()?;
        let pair = (input.mint, output.mint);
        let paths = self.path_lookups(pair)?;
        if paths.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let (live, skipped) = self.live_paths(&paths, pair, value);
        let route = if skipped && !live.is_empty() {
            match self.quote_paths(request, &input, &output, live).await {
                Err(SwapperError::NoQuoteAvailable) => self.quote_paths(request, &input, &output, paths).await,
                route => route,
            }
        } else {
            self.quote_paths(request, &input, &output, paths).await
        }?;
        Quote::from_route(request, &self.provider, BigUint::from(route.received_value()), &route, None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route: DexRoute = quote.route_data()?;
        let input = SwapToken::from_asset(&quote.request.from_asset)?;
        let output = SwapToken::from_asset(&quote.request.to_asset)?;
        let known_pools = route
            .hops
            .iter()
            .map(|hop| Ok(self.protocol.pool_addresses(&hop.input_mint, &hop.output_mint)?.contains(&hop.pool)))
            .collect::<Result<Vec<_>, SwapperError>>()?;
        if known_pools.contains(&false) || BigUint::from(route.input_value()) != quote.from_value {
            return Err(SwapperError::InvalidRoute);
        }
        let authority = Pubkey::from_base58(&quote.request.wallet_address).map_err(|_| SwapperError::transaction_error(INVALID_ADDRESS))?;
        let (pools, blockhash) = self.fetch_pools_and_blockhash(route.hops.iter().map(|hop| hop.pool).collect()).await?;
        let transaction = self.transaction(&route, &pools, &input, &output, authority, quote.data.slippage_bps)?;

        Ok(SwapperQuoteData::new_contract(
            P::PROGRAM_ID.to_base58(),
            BigUint::ZERO,
            transaction.encode(&blockhash)?,
            None,
            Some(DEFAULT_SWAP_GAS_LIMIT.to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{SYSTEM_PROGRAM_ID, TOKEN_PROGRAM};
    use primitives::swap::SOLANA_REFERRAL_ADDRESS;

    #[test]
    fn test_referral_fee_side() {
        let referral = Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap();
        let input = SwapToken::mock();
        let output = SwapToken {
            mint: Pubkey::new([7; 32]),
            ..SwapToken::mock()
        };
        let token_account = Some(AccountData {
            data: vec![],
            owner: TOKEN_PROGRAM.to_string(),
        });
        let system_account = Some(AccountData {
            data: vec![],
            owner: SYSTEM_PROGRAM_ID.to_string(),
        });
        let side = |input_account: Option<AccountData>, output_account: Option<AccountData>| {
            let accounts = Accounts::from([(input.account(&referral).unwrap(), input_account), (output.account(&referral).unwrap(), output_account)]);
            referral_fee_side(&accounts, &referral, &input, &output)
        };

        assert_eq!(side(token_account.clone(), token_account.clone()), Ok(FeeSide::Input));
        assert_eq!(side(None, token_account.clone()), Ok(FeeSide::Output));
        assert_eq!(side(system_account, token_account), Ok(FeeSide::Output));
        assert_eq!(side(None, None), Err(SwapperError::compute_quote_error("Referral fee account is unavailable")));
    }
}
