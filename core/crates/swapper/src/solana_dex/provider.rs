use async_trait::async_trait;
use borsh::BorshDeserialize;
use futures::future::{join_all, try_join_all};
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use gem_solana::{
    Pubkey, SolanaRpc,
    instructions::program_ids::{sysvar_clock, sysvar_program, token_2022_program, token_program},
    models::{AccountData, Clock, ProgramAccount, TokenAccount, ValueResult},
};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, DAY};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt,
    sync::Arc,
};

use super::{
    protocol::{DexProtocol, PoolDepth, PoolQuery, PoolSwap, QuoteAccount},
    route::{DexHop, DexRoute, preferred_fee_side},
    token::{SwapToken, decode_mint},
    transaction::DexTransaction,
};
use crate::{
    Contracts, FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperQuoteData,
    client_factory::create_client_with_chain,
    error::INVALID_ADDRESS,
    fees::{FeeSide, default_referral_fees, on_chain_referral_fees, subtract_bps},
    route_cache::{Cache, DiscoveryCache, OutpricedCache},
    route_path::RoutePath,
};

type AccountsResult = ValueResult<Vec<Option<AccountData>>>;
type Accounts = HashMap<Pubkey, Option<AccountData>>;
type Pair = (Pubkey, Pubkey);
type PoolRoute<Pool> = Vec<(DexHop, Pool)>;
type RouteOutput = (u64, Option<u64>);

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

const MAX_QUERIED_POOLS: usize = 4;
const MAX_ACCOUNTS_PER_REQUEST: usize = 100;

pub(crate) struct SolanaDex<P: DexProtocol, C: Client + Clone = RpcClient> {
    provider: ProviderType,
    protocol: P,
    client: JsonRpcClient<C>,
    pools: DiscoveryCache<(Pubkey, P::Pool), Pubkey, Pubkey>,
    queried_pools: DiscoveryCache<Vec<Pubkey>, (), Pubkey>,
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
            queried_pools: DiscoveryCache::with_found_ttl(DAY),
            constant_accounts: Cache::default(),
            outpriced_routes: OutpricedCache::default(),
        }
    }

    fn route_paths(&self, (input_mint, output_mint): Pair) -> Vec<RoutePath<Pubkey>> {
        RoutePath::all(&input_mint, &output_mint, self.protocol.intermediate_tokens().iter().map(|token| token.mint), |left, right| left == right)
    }

    fn path_lookups(&self, (input_mint, output_mint): Pair) -> Result<Vec<PathLookup<P::Pool>>, SwapperError> {
        let lookups = self
            .route_paths((input_mint, output_mint))
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
            pools_only_in(&outpriced, &live)
        }
    }

    fn requested_accounts(&self, paths: &[PathLookup<P::Pool>], pair: Pair) -> Vec<Pubkey> {
        let lookups = paths.iter().flat_map(|(_, lookups)| lookups).collect::<Vec<_>>();
        let known = lookups.iter().map(|lookup| (lookup.pair, lookup.known.clone())).collect();
        let predicted = self.predicted_quote_accounts(&self.routes(paths.iter().map(|(path, _)| path), pair, &known));
        lookups.iter().flat_map(|lookup| lookup.addresses.clone()).chain(predicted).collect()
    }

    fn pool_probes(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<Vec<Pubkey>, SwapperError> {
        let queried = self.queried_pools.candidates_for_probes(*input_mint, *output_mint, &[()]).into_iter().flatten();
        Ok(self.protocol.pool_addresses(input_mint, output_mint)?.into_iter().chain(queried).collect())
    }

    async fn query_route_pools(&self, pair: Pair) -> Result<(), SwapperError> {
        let pairs = self.route_paths(pair).iter().flat_map(|path| path.pairs(&pair.0, &pair.1)).collect::<BTreeSet<_>>();
        let results = join_all(pairs.iter().map(|pair| self.query_pools(*pair))).await;
        pairs.into_iter().zip(results).find(|(queried, _)| *queried == pair).map_or(Ok(()), |(_, result)| result)
    }

    async fn query_pools(&self, (input_mint, output_mint): Pair) -> Result<(), SwapperError> {
        let Some(query) = self.protocol.pool_query(&input_mint, &output_mint) else {
            return Ok(());
        };
        if self.queried_pools.missing_probes(input_mint, output_mint, &[()]).is_empty() {
            return Ok(());
        }
        let pools = deepest_pools(self.fetch_pool_depths(query).await?);
        self.queried_pools.record_discovery(input_mint, output_mint, [((), (!pools.is_empty()).then_some(pools))]);
        Ok(())
    }

    async fn fetch_pool_depths(&self, query: PoolQuery) -> Result<Vec<(Pubkey, u128)>, SwapperError> {
        let slice = query.depth;
        let requests = query
            .filters
            .into_iter()
            .map(|filters| self.client.request::<Vec<ProgramAccount>, _>(SolanaRpc::GetProgramAccountsSlice(P::PROGRAM_ID.to_base58(), filters, slice)));
        let pools = try_join_all(requests)
            .await?
            .into_iter()
            .flatten()
            .map(|account| Ok((Pubkey::from_base58(&account.pubkey)?, self.protocol.pool_depth(&account.account.decoded_data()?))))
            .collect::<Result<Vec<_>, SwapperError>>()?;
        let reserves = pools
            .iter()
            .flat_map(|(_, depth)| match depth {
                PoolDepth::Reserves(reserve_a, reserve_b) => vec![*reserve_a, *reserve_b],
                PoolDepth::Value(_) => vec![],
            })
            .collect::<Vec<_>>();
        let amounts = self.fetch_token_amounts(reserves).await?;
        let amount = |reserve: &Pubkey| u128::from(amounts.get(reserve).copied().unwrap_or_default());
        Ok(pools
            .into_iter()
            .map(|(address, depth)| match depth {
                PoolDepth::Value(value) => (address, value),
                PoolDepth::Reserves(reserve_a, reserve_b) => (address, amount(&reserve_a) * amount(&reserve_b)),
            })
            .collect())
    }

    async fn fetch_token_amounts(&self, accounts: Vec<Pubkey>) -> Result<HashMap<Pubkey, u64>, SwapperError> {
        let requests = accounts.chunks(MAX_ACCOUNTS_PER_REQUEST).map(|chunk| async move {
            let response: AccountsResult = self.client.request(SolanaRpc::GetMultipleAccountsSlice(chunk.iter().map(Pubkey::to_base58).collect(), TokenAccount::AMOUNT_SLICE)).await?;
            Ok::<_, SwapperError>(chunk.iter().copied().zip(response.value).collect::<Vec<_>>())
        });
        Ok(try_join_all(requests)
            .await?
            .into_iter()
            .flatten()
            .filter_map(|(address, account)| Some((address, TokenAccount::decode_amount(&account?.decoded_data().ok()?).ok()?)))
            .collect())
    }

    fn pool_lookup(&self, pair: Pair) -> Result<PoolLookup<P::Pool>, SwapperError> {
        let (input_mint, output_mint) = pair;
        let probes = self.pool_probes(&input_mint, &output_mint)?;
        let known = self.pools.candidates_for_probes(input_mint, output_mint, &probes);
        let addresses = known.iter().map(|(address, _)| *address).chain(self.pools.missing_probes(input_mint, output_mint, &probes)).collect();
        Ok(PoolLookup { pair, known, addresses })
    }

    fn record_pools(&self, lookup: &PoolLookup<P::Pool>, accounts: &Accounts, clock: &Clock) -> Vec<(Pubkey, P::Pool)> {
        let (input_mint, output_mint) = lookup.pair;
        let (usable, unusable): (Vec<_>, Vec<_>) = lookup
            .addresses
            .iter()
            .map(|address| (*address, self.decode_pool(accounts, address)))
            .partition(|(_, pool)| pool.as_ref().is_none_or(|(_, pool)| self.protocol.is_tradable(pool, clock)));
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
            .flat_map(|path| {
                let legs = path.pairs(&input_mint, &output_mint).into_iter().map(|pair| hops(pair).collect::<Vec<_>>()).collect::<Vec<_>>();
                let native_two_hop = self.protocol.native_two_hop().filter(|_| legs.len() > 1);
                legs.into_iter()
                    .map(|leg| match native_two_hop {
                        Some(two_hop) => leg.into_iter().max_by_key(|(_, pool)| two_hop.liquidity(pool)).into_iter().collect(),
                        None => leg,
                    })
                    .fold(vec![vec![]], |routes: Vec<PoolRoute<P::Pool>>, leg| {
                        routes.iter().flat_map(|route| leg.iter().map(move |hop| [route.clone(), vec![hop.clone()]].concat())).collect()
                    })
            })
            .collect()
    }

    fn route_quote_accounts(&self, route: &[(DexHop, P::Pool)]) -> Option<Vec<Vec<QuoteAccount>>> {
        route.iter().map(|(hop, pool)| self.protocol.quote_accounts(&hop.pool, pool, &hop.input_mint).ok()).collect()
    }

    fn predicted_quote_accounts(&self, routes: &[PoolRoute<P::Pool>]) -> Vec<Pubkey> {
        routes.iter().filter_map(|route| self.route_quote_accounts(route)).flatten().flatten().map(|account| account.address()).collect()
    }

    async fn quote_routes(&self, routes: Vec<PoolRoute<P::Pool>>, amount_in: u64, slippage_bps: u32, accounts: Accounts, clock: &Clock) -> Result<Vec<(PoolRoute<P::Pool>, Option<RouteOutput>)>, SwapperError> {
        let route_accounts = routes.iter().map(|route| self.route_quote_accounts(route)).collect::<Vec<_>>();
        let missing = route_accounts.iter().flatten().flatten().flatten().map(QuoteAccount::address).filter(|address| !accounts.contains_key(address)).collect();
        let constants = self.constant_route_accounts(routes.iter().flatten().map(|(_, pool)| pool));
        let accounts = accounts.into_iter().chain(self.load_accounts(missing, &constants).await?).collect::<Accounts>();
        Ok(routes
            .into_iter()
            .zip(route_accounts)
            .map(|(route, hop_accounts)| {
                let output = hop_accounts.and_then(|hop_accounts| {
                    route.iter().zip(hop_accounts).enumerate().try_fold((amount_in, None), |(amount, intermediate_value), (index, ((hop, pool), quote_accounts))| {
                        let chained = index > 0 && self.protocol.native_two_hop().is_none();
                        let amount = if chained { subtract_bps(&amount, slippage_bps) } else { amount };
                        let data = quote_accounts.iter().map(|account| quote_data(&accounts, account)).collect::<Result<Vec<_>, _>>().ok()?;
                        let value = self.protocol.quote(&hop.pool, pool, &hop.input_mint, amount, &data, clock).ok()?;
                        Some((value, if chained { Some(amount) } else { intermediate_value }))
                    })
                });
                (route, output)
            })
            .collect())
    }

    async fn quote_paths(&self, request: &QuoteRequest, input: &SwapToken, output: &SwapToken, paths: Vec<PathLookup<P::Pool>>) -> Result<DexRoute, SwapperError> {
        let value = request.value_u64()?;
        let referral_fee = on_chain_referral_fees(request).solana;
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
        let pools = paths.iter().flat_map(|(_, lookups)| lookups).map(|lookup| (lookup.pair, self.record_pools(lookup, &accounts, &clock))).collect();
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
        let quotes = self.quote_routes(routes, pool_input, request.options.slippage.bps, accounts, &clock).await?;
        self.outpriced_routes.record(
            &input.mint,
            &output.mint,
            u128::from(value),
            quotes.iter().map(|(route, output)| (route_pools(route), output.map_or(0, |(value, _)| u128::from(value)))),
        );
        ranked(quotes)
            .into_iter()
            .map(|(pools, (pool_output, intermediate_value))| {
                let output_value = pool_output.saturating_sub(output_mint.transfer_fee(clock.epoch, pool_output));
                let route = DexRoute {
                    hops: pools.iter().map(|(hop, _)| *hop).collect(),
                    swap_value,
                    output_value,
                    fee: referral_fee.transfer(fee_side, value, output_value),
                    intermediate_value,
                };
                (pools.into_iter().map(|(hop, pool)| (hop.pool, pool)).collect::<Vec<_>>(), route)
            })
            .find(|(pools, route)| route.received_value() > 0 && self.transaction(route, pools, input, output, authority, request.options.slippage.bps).and_then(|transaction| transaction.encode()).is_ok())
            .map(|(_, route)| route)
            .ok_or(SwapperError::NoQuoteAvailable)
    }

    fn transaction<'a>(&self, route: &DexRoute, pools: &[(Pubkey, P::Pool)], input: &'a SwapToken, output: &'a SwapToken, authority: Pubkey, slippage_bps: u32) -> Result<DexTransaction<'a>, SwapperError> {
        let swap = PoolSwap {
            authority,
            input,
            intermediate: route.intermediate(&input.mint, &output.mint, self.protocol.intermediate_tokens())?,
            output,
            amount: route.swap_value,
            minimum_out: route.minimum_output(slippage_bps),
        };
        let swap_instructions = match (pools, swap.intermediate, self.protocol.native_two_hop()) {
            ([(address, pool)], None, _) => vec![self.protocol.swap_instruction(address, pool, &swap)?],
            ([first, second], Some(intermediate), Some(two_hop)) => vec![two_hop.two_hop_swap_instruction(first, second, &swap, intermediate)?],
            ([(first_address, first_pool), (second_address, second_pool)], Some(intermediate), None) => {
                let intermediate_value = route.intermediate_value.ok_or(SwapperError::InvalidRoute)?;
                let first = PoolSwap {
                    intermediate: None,
                    output: intermediate,
                    minimum_out: intermediate_value,
                    ..swap
                };
                let second = PoolSwap {
                    input: intermediate,
                    intermediate: None,
                    amount: intermediate_value,
                    ..swap
                };
                vec![self.protocol.swap_instruction(first_address, first_pool, &first)?, self.protocol.swap_instruction(second_address, second_pool, &second)?]
            }
            _ => return Err(SwapperError::InvalidRoute),
        };
        Ok(DexTransaction {
            compute_unit_limit: self.protocol.compute_unit_limit(swap_instructions.len()),
            swap_instructions,
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

    async fn fetch_pools(&self, addresses: Vec<Pubkey>) -> Result<Vec<(Pubkey, P::Pool)>, SwapperError> {
        let accounts = self.fetch_accounts(addresses.clone()).await?;
        addresses
            .into_iter()
            .map(|address| {
                let data = Self::program_data(&accounts, &address)?.ok_or(SwapperError::InvalidRoute)?;
                Ok((address, self.protocol.decode_pool(&data)?))
            })
            .collect()
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

fn deepest_pools(pools: Vec<(Pubkey, u128)>) -> Vec<Pubkey> {
    let ranked = pools.into_iter().filter(|(_, depth)| *depth > 0).map(|(address, depth)| (Reverse(depth), address)).collect::<BTreeSet<_>>();
    ranked.into_iter().take(MAX_QUERIED_POOLS).map(|(_, address)| address).collect()
}

fn has_pools<Pool>(lookups: &[PoolLookup<Pool>]) -> bool {
    lookups.iter().all(|lookup| !lookup.addresses.is_empty())
}

fn route_pools<Pool>(route: &[(DexHop, Pool)]) -> Vec<Pubkey> {
    route.iter().map(|(hop, _)| hop.pool).collect()
}

fn pools_only_in<Pool>(routes: &[PoolRoute<Pool>], others: &[PoolRoute<Pool>]) -> Vec<Pubkey> {
    let kept = others.iter().flat_map(|route| route_pools(route)).collect::<HashSet<_>>();
    routes.iter().flat_map(|route| route_pools(route)).filter(|pool| !kept.contains(pool)).collect()
}

fn ranked<Pool>(quotes: Vec<(PoolRoute<Pool>, Option<RouteOutput>)>) -> Vec<(PoolRoute<Pool>, RouteOutput)> {
    let ranked = quotes
        .into_iter()
        .enumerate()
        .filter_map(|(index, (route, output))| Some(((Reverse(output?.0), route.len(), index), (route, output?))))
        .collect::<BTreeMap<_, _>>();
    ranked.into_values().collect()
}

fn owned_account<'a>(accounts: &'a Accounts, address: &Pubkey, owner: &Pubkey) -> Option<&'a AccountData> {
    accounts.get(address).and_then(Option::as_ref).filter(|account| account.owner == owner.to_base58())
}

fn owned_data(accounts: &Accounts, address: &Pubkey, owner: &Pubkey) -> Result<Option<Vec<u8>>, SwapperError> {
    Ok(owned_account(accounts, address, owner).map(AccountData::decoded_data).transpose()?)
}

fn quote_data(accounts: &Accounts, account: &QuoteAccount) -> Result<Option<Vec<u8>>, SwapperError> {
    match account {
        QuoteAccount::Owned { address, owner } => owned_data(accounts, address, owner),
        QuoteAccount::Token(address) => Ok([token_program(), token_2022_program()]
            .iter()
            .find_map(|owner| owned_account(accounts, address, owner))
            .map(AccountData::decoded_data)
            .transpose()?),
    }
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

    fn contracts(&self, _from_asset: &AssetId) -> Contracts {
        Contracts::new(vec![P::PROGRAM])
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let input = SwapToken::from_asset(&request.from_asset)?;
        let output = SwapToken::from_asset(&request.to_asset)?;
        let value = request.value_u64()?;
        let pair = (input.mint, output.mint);
        self.query_route_pools(pair).await?;
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
        Quote::from_route(request, &self.provider, BigUint::from(route.received_value()), request.options.slippage.bps, &route, None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route: DexRoute = quote.route_data()?;
        let input = SwapToken::from_asset(&quote.request.from_asset)?;
        let output = SwapToken::from_asset(&quote.request.to_asset)?;
        let known_pools = route
            .hops
            .iter()
            .map(|hop| Ok(self.pool_probes(&hop.input_mint, &hop.output_mint)?.contains(&hop.pool)))
            .collect::<Result<Vec<_>, SwapperError>>()?;
        if known_pools.contains(&false) || BigUint::from(route.input_value()) != quote.from_value {
            return Err(SwapperError::InvalidRoute);
        }
        let authority = Pubkey::from_base58(&quote.request.wallet_address).map_err(|_| SwapperError::transaction_error(INVALID_ADDRESS))?;
        let pools = self.fetch_pools(route.hops.iter().map(|hop| hop.pool).collect()).await?;
        let transaction = self.transaction(&route, &pools, &input, &output, authority, quote.data.slippage_bps)?;

        Ok(SwapperQuoteData::new_contract(
            P::PROGRAM_ID.to_base58(),
            BigUint::ZERO,
            transaction.encode()?,
            None,
            Some(transaction.compute_unit_limit.to_string()),
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

    #[test]
    fn test_pools_only_in_keeps_pools_a_winning_route_still_uses() {
        let pool = |byte: u8| Pubkey::new([byte; 32]);
        let route = |pools: &[u8]| {
            pools
                .iter()
                .map(|byte| {
                    (
                        DexHop {
                            pool: pool(*byte),
                            input_mint: Pubkey::new([0; 32]),
                            output_mint: Pubkey::new([0; 32]),
                        },
                        (),
                    )
                })
                .collect::<Vec<_>>()
        };
        let live = [route(&[1, 3])];
        let outpriced = [route(&[2, 3]), route(&[1, 4]), route(&[5])];

        assert_eq!(pools_only_in(&outpriced, &live), vec![pool(2), pool(4), pool(5)]);
        assert_eq!(pools_only_in(&outpriced, &[]), vec![pool(2), pool(3), pool(1), pool(4), pool(5)]);
    }

    #[test]
    fn test_deepest_pools() {
        let pool = |byte: u8| Pubkey::new([byte; 32]);
        let depths = vec![(pool(1), 5), (pool(2), 0), (pool(3), 9), (pool(4), 7), (pool(5), 1), (pool(6), 3)];

        assert_eq!(deepest_pools(depths), vec![pool(3), pool(4), pool(1), pool(6)]);
        assert_eq!(deepest_pools(vec![(pool(1), 0)]), vec![]);
    }

    #[test]
    fn test_pool_depth_reads_little_endian_of_any_width() {
        let depth = |slice: &[u8]| crate::meteora_damm_v2::MeteoraDammV2.pool_depth(slice);

        assert_eq!(depth(&7u64.to_le_bytes()), PoolDepth::Value(7));
        assert_eq!(depth(&(u128::from(u64::MAX) + 2).to_le_bytes()), PoolDepth::Value(u128::from(u64::MAX) + 2));
        assert_eq!(depth(&[]), PoolDepth::Value(0));
    }
}
