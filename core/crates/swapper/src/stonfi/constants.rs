use gem_ton::constants::TON_PROXY_JETTON_ADDRESS;
use primitives::asset_constants::TON_USDT_TOKEN_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RouterInfo {
    pub address: &'static str,
    pub major_version: u8,
    pub minor_version: u8,
    pub pton_wallet: &'static str,
}

impl RouterInfo {
    pub(super) fn is_supported_v2(&self) -> bool {
        self.major_version == 2 && (self.minor_version == 1 || self.minor_version == 2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StaticPool {
    pub token0: &'static str,
    pub token1: &'static str,
    pub pool_address: &'static str,
    pub router: RouterInfo,
    pub token0_wallet: &'static str,
    pub token1_wallet: &'static str,
    pub lp_fee_bps: Option<u32>,
}

#[rustfmt::skip]
const PRIMARY_ROUTER: RouterInfo = router("EQCS4UEa5UaJLzOyyKieqQOQ2P9M-7kXpkO5HnP3Bv250cN3", 2, 2, "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3");
#[rustfmt::skip]
const V1_ROUTER: RouterInfo = router("EQB3ncyBUTjZUA5EnFKR5_EnOMI9V1tTEAAPaiU71gc4TiUt", 1, 0, "EQARULUYsmJq1RiZ-YiH-IJLcAZUVkVff-KBPwEmmaQGH6aC");
#[rustfmt::skip]
const NOT_TON_ROUTER: RouterInfo = router("EQDx--jUU9PUtHltPYZX7wdzIi0SPY3KZ8nvOs0iZvQJd6Ql", 2, 2, "EQDwOyDlewGw8MkeXgZ_oOmPTIhJIlaJwhJmf4ffIPKv-294");

#[rustfmt::skip]
pub(super) const FALLBACK_ROUTERS: &[RouterInfo] = &[PRIMARY_ROUTER, V1_ROUTER];

pub(super) const STATIC_POOLS: &[StaticPool] = &[
    StaticPool {
        token0: TON_PROXY_JETTON_ADDRESS,
        token1: TON_USDT_TOKEN_ID,
        pool_address: "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
        router: PRIMARY_ROUTER,
        token0_wallet: "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3",
        token1_wallet: "EQCSLWJ9fY7b0A5OI72wxUp27l4fRlc6GvRBeFf6PiPpH4p3",
        lp_fee_bps: Some(7),
    },
    StaticPool {
        token0: TON_PROXY_JETTON_ADDRESS,
        token1: TON_USDT_TOKEN_ID,
        pool_address: "EQD8TJ8xEWB1SpnRE4d89YO3jl0W0EiBnNS4IBaHaUmdfizE",
        router: V1_ROUTER,
        token0_wallet: "EQARULUYsmJq1RiZ-YiH-IJLcAZUVkVff-KBPwEmmaQGH6aC",
        token1_wallet: "EQBO7JIbnU1WoNlGdgFtScJrObHXkBp-FT5mAz8UagiG9KQR",
        lp_fee_bps: Some(20),
    },
    StaticPool {
        token0: "EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT",
        token1: TON_PROXY_JETTON_ADDRESS,
        pool_address: "EQD9BmgQQ2_nzk-9LfxthcoLYC3yBHWK5WqEv_FyMU2riRvE",
        router: NOT_TON_ROUTER,
        token0_wallet: "EQAZMdggoCwOcSVLlT_RyiZtLSMyjYHIttUD9QBVe_NjIHA4",
        token1_wallet: "EQDwOyDlewGw8MkeXgZ_oOmPTIhJIlaJwhJmf4ffIPKv-294",
        lp_fee_bps: Some(20),
    },
];

const fn router(address: &'static str, major_version: u8, minor_version: u8, pton_wallet: &'static str) -> RouterInfo {
    RouterInfo {
        address,
        major_version,
        minor_version,
        pton_wallet,
    }
}
