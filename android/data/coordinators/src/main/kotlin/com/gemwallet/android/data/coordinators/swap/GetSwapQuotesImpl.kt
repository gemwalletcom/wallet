package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapQuotes
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemSwapper
import uniffi.gemstone.SwapperOptions
import uniffi.gemstone.SwapperQuote
import uniffi.gemstone.SwapperQuoteAsset
import uniffi.gemstone.SwapperQuoteRequest
import uniffi.gemstone.SwapperSlippage
import uniffi.gemstone.SwapperSlippageMode
import uniffi.gemstone.getDefaultSlippage
import java.math.BigInteger

class GetSwapQuotesImpl(
    private val gemSwapper: GemSwapper,
) : GetSwapQuotes {
    override suspend fun getQuotes(
        ownerAddress: String,
        destination: String,
        from: Asset,
        to: Asset,
        amount: String,
        useMaxAmount: Boolean,
        slippageBps: UInt?,
    ): List<SwapperQuote> {
        val slippage = slippageBps?.let { SwapperSlippage(bps = it, mode = SwapperSlippageMode.EXACT) }
            ?: getDefaultSlippage(from.chain.string).copy(mode = SwapperSlippageMode.AUTO)
        val swapRequest = SwapperQuoteRequest(
            fromAsset = SwapperQuoteAsset(
                id = from.id.toIdentifier(),
                symbol = from.symbol,
                decimals = from.decimals.toUInt(),
            ),
            toAsset = SwapperQuoteAsset(
                id = to.id.toIdentifier(),
                symbol = to.symbol,
                decimals = to.decimals.toUInt(),
            ),
            walletAddress = ownerAddress,
            destinationAddress = destination,
            value = amount,
            options = SwapperOptions(
                slippage = slippage,
                useMaxAmount = useMaxAmount,
            )
        )
        return gemSwapper.getQuote(swapRequest)
            .sortedByDescending { BigInteger(it.toValue) }
    }
}
