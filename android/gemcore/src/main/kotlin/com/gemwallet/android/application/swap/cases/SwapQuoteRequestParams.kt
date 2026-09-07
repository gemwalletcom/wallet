package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import java.math.BigDecimal
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Crypto
import uniffi.gemstone.GemSwapRequest
import java.math.BigInteger

data class SwapQuoteRequestParams(
    val value: BigDecimal,
    val pay: AssetInfo,
    val receive: AssetInfo,
    val slippageBps: UInt? = null,
) {
    val key: SwapQuoteRequestKey
        get() = SwapQuoteRequestKey(Crypto(value, pay.asset.decimals).atomicValue, pay.id(), receive.id(), slippageBps)

    companion object
}

data class SwapQuoteRequestKey(
    val value: BigInteger,
    val payAssetId: AssetId,
    val receiveAssetId: AssetId,
    val slippageBps: UInt? = null,
)

fun SwapQuoteRequestKey.toGem(): GemSwapRequest = GemSwapRequest(
    payAssetId = payAssetId.toIdentifier(),
    receiveAssetId = receiveAssetId.toIdentifier(),
    value = value,
    slippageBps = slippageBps,
)

fun SwapQuoteRequestParams.Companion.create(value: BigDecimal, pay: AssetInfo?, receive: AssetInfo?, slippageBps: UInt? = null): SwapQuoteRequestParams? {
    return if (pay == null || receive == null || pay.id() == receive.id() || value.compareTo(BigDecimal.ZERO) == 0) {
        null
    } else {
        SwapQuoteRequestParams(value, pay, receive, slippageBps)
    }
}
