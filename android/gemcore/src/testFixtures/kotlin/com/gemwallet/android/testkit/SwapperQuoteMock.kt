package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.SwapperOptions
import uniffi.gemstone.SwapperProviderData
import uniffi.gemstone.SwapperProviderMode
import uniffi.gemstone.SwapperProviderType
import uniffi.gemstone.SwapperQuote
import uniffi.gemstone.SwapperQuoteAsset
import uniffi.gemstone.SwapperQuoteRequest
import uniffi.gemstone.SwapperRoute
import uniffi.gemstone.SwapperSlippage
import uniffi.gemstone.SwapperSlippageMode
import java.math.BigInteger

fun mockSwapperQuote(toValue: BigInteger = BigInteger("2500000")): SwapperQuote {
    val fromValue = BigInteger("1000000000")
    val fromAsset = mockAssetSolana()
    val toAsset = mockAssetSolanaUSDC()
    return SwapperQuote(
        fromValue = fromValue,
        minFromValue = null,
        toValue = toValue,
        data = SwapperProviderData(
            provider = SwapperProviderType(
                id = SwapProvider.UNISWAP_V3,
                name = "Uniswap",
                protocol = "v3",
                protocolId = "uniswap_v3",
                mode = SwapperProviderMode.OnChain,
                slippageMode = SwapperSlippageMode.EXACT,
            ),
            slippageBps = 50u,
            routes = listOf(
                SwapperRoute(
                    input = fromAsset.id.toIdentifier(),
                    output = toAsset.id.toIdentifier(),
                    routeData = "0x",
                ),
            ),
        ),
        request = SwapperQuoteRequest(
            fromAsset = SwapperQuoteAsset(
                id = fromAsset.id.toIdentifier(),
                symbol = fromAsset.symbol,
                decimals = fromAsset.decimals.toUInt(),
                assetType = fromAsset.type.toGem(),
            ),
            toAsset = SwapperQuoteAsset(
                id = toAsset.id.toIdentifier(),
                symbol = toAsset.symbol,
                decimals = toAsset.decimals.toUInt(),
                assetType = toAsset.type.toGem(),
            ),
            walletAddress = mockAccount(chain = fromAsset.id.chain).address,
            destinationAddress = mockAccount(chain = toAsset.id.chain).address,
            value = fromValue,
            options = SwapperOptions(
                slippage = SwapperSlippage(bps = 50u, mode = SwapperSlippageMode.AUTO),
                useMaxAmount = false,
            ),
        ),
        etaInSeconds = 30u,
    )
}
