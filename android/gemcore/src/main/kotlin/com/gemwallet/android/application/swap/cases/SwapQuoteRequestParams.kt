package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import java.math.BigDecimal
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Crypto
import uniffi.gemstone.GemSwapRequest

data class SwapQuoteRequestParams(
    val value: BigDecimal,
    val pay: AssetInfo,
    val receive: AssetInfo,
    val slippageBps: UInt? = null,
) {
    val key: GemSwapRequest
        get() = GemSwapRequest(
            payAssetId = pay.id().toIdentifier(),
            receiveAssetId = receive.id().toIdentifier(),
            value = Crypto(value, pay.asset.decimals).atomicValue,
            slippageBps = slippageBps,
        )

    companion object
}

fun SwapQuoteRequestParams.Companion.create(value: BigDecimal, pay: AssetInfo?, receive: AssetInfo?, slippageBps: UInt? = null): SwapQuoteRequestParams? {
    return if (pay == null || receive == null || pay.id() == receive.id() || value.compareTo(BigDecimal.ZERO) == 0) {
        null
    } else {
        SwapQuoteRequestParams(value, pay, receive, slippageBps)
    }
}
