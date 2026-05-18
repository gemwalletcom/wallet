package com.gemwallet.android.blockchain.gemstone

import com.wallet.core.primitives.SwapProvider
import uniffi.gemstone.SwapperProvider

internal fun SwapProvider.toGem(): SwapperProvider = when (this) {
    SwapProvider.UniswapV3 -> SwapperProvider.UNISWAP_V3
    SwapProvider.UniswapV4 -> SwapperProvider.UNISWAP_V4
    SwapProvider.PancakeswapV3 -> SwapperProvider.PANCAKESWAP_V3
    SwapProvider.Aerodrome -> SwapperProvider.AERODROME
    SwapProvider.Panora -> SwapperProvider.PANORA
    SwapProvider.Thorchain -> SwapperProvider.THORCHAIN
    SwapProvider.Jupiter -> SwapperProvider.JUPITER
    SwapProvider.Okx -> SwapperProvider.OKX
    SwapProvider.Across -> SwapperProvider.ACROSS
    SwapProvider.Oku -> SwapperProvider.OKU
    SwapProvider.Wagmi -> SwapperProvider.WAGMI
    SwapProvider.StonfiV2 -> SwapperProvider.STONFI_V2
    SwapProvider.Mayan -> SwapperProvider.MAYAN
    SwapProvider.Chainflip -> SwapperProvider.CHAINFLIP
    SwapProvider.NearIntents -> SwapperProvider.NEAR_INTENTS
    SwapProvider.CetusAggregator -> SwapperProvider.CETUS_AGGREGATOR
    SwapProvider.CetusClmm -> SwapperProvider.CETUS_CLMM
    SwapProvider.Relay -> SwapperProvider.RELAY
    SwapProvider.Hyperliquid -> SwapperProvider.HYPERLIQUID
    SwapProvider.Orca -> SwapperProvider.ORCA
    SwapProvider.Squid -> SwapperProvider.SQUID
}
