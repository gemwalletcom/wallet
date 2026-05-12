package com.gemwallet.android.application.swap.coordinators

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import java.math.BigDecimal

data class SwapQuoteRequestParams(
    val value: BigDecimal,
    val pay: AssetInfo,
    val receive: AssetInfo,
) {
    val key: SwapQuoteRequestKey
        get() = SwapQuoteRequestKey(value, pay.id(), receive.id())

    companion object
}

class SwapQuoteRequestKey(
    val value: BigDecimal,
    val payAssetId: AssetId,
    val receiveAssetId: AssetId,
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
            receiveAssetId == other.receiveAssetId
    }

    override fun hashCode(): Int {
        var result = value.stripTrailingZeros().hashCode()
        result = 31 * result + payAssetId.hashCode()
        result = 31 * result + receiveAssetId.hashCode()
        return result
    }
}

fun SwapQuoteRequestParams.Companion.create(value: BigDecimal, pay: AssetInfo?, receive: AssetInfo?): SwapQuoteRequestParams? {
    return if (pay == null || receive == null || pay.id() == receive.id() || value.compareTo(BigDecimal.ZERO) == 0) {
        null
    } else {
        SwapQuoteRequestParams(value, pay, receive)
    }
}
