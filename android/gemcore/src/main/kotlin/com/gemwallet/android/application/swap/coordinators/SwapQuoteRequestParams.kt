package com.gemwallet.android.application.swap.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import java.math.BigDecimal

data class SwapQuoteRequestParams(
    val value: BigDecimal,
    val pay: AssetInfo,
    val receive: AssetInfo,
    val slippageBps: UInt? = null,
) {
    val key: SwapQuoteRequestKey
        get() = SwapQuoteRequestKey(value, pay.id(), receive.id(), slippageBps)

    companion object
}

class SwapQuoteRequestKey(
    val value: BigDecimal,
    val payAssetId: AssetId,
    val receiveAssetId: AssetId,
    val slippageBps: UInt? = null,
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) {
            return true
        }
        if (other !is SwapQuoteRequestKey) {
            return false
        }

        return value.compareTo(other.value) == 0 &&
            payAssetId == other.payAssetId &&
            receiveAssetId == other.receiveAssetId &&
            slippageBps == other.slippageBps
    }

    override fun hashCode(): Int {
        var result = value.stripTrailingZeros().hashCode()
        result = 31 * result + payAssetId.hashCode()
        result = 31 * result + receiveAssetId.hashCode()
        result = 31 * result + (slippageBps?.hashCode() ?: 0)
        return result
    }
}

fun SwapQuoteRequestParams.Companion.create(value: BigDecimal, pay: AssetInfo?, receive: AssetInfo?, slippageBps: UInt? = null): SwapQuoteRequestParams? {
    return if (pay == null || receive == null || pay.id() == receive.id() || value.compareTo(BigDecimal.ZERO) == 0) {
        null
    } else {
        SwapQuoteRequestParams(value, pay, receive, slippageBps)
    }
}
