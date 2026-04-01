package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import java.math.BigDecimal

internal data class QuoteRequestParams(
    val value: BigDecimal,
    val pay: AssetInfo,
    val receive: AssetInfo,
) {
    val key: QuoteRequestKey
        get() = QuoteRequestKey(value, pay.id(), receive.id())

    companion object
}

internal class QuoteRequestKey(
    val value: BigDecimal,
    val payAssetId: AssetId,
    val receiveAssetId: AssetId,
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) {
            return true
        }
        if (other !is QuoteRequestKey) {
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

internal fun QuoteRequestParams.Companion.create(value: BigDecimal, pay: AssetInfo?, receive: AssetInfo?): QuoteRequestParams? {
    return if (pay == null || receive == null || pay.id() == receive.id() || value.compareTo(BigDecimal.ZERO) == 0) {
        null
    } else {
        QuoteRequestParams(value, pay, receive)
    }
}
