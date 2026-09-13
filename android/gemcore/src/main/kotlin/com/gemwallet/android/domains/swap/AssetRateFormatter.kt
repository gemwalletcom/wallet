package com.gemwallet.android.domains.swap

import com.gemwallet.android.model.NumericFormatter
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemSwapRate
import java.util.Locale

data class AssetRatePair(
    val forward: String,
    val reverse: String,
)

class AssetRateFormatter(
    locale: Locale = Locale.getDefault(),
) {
    private val formatter = NumericFormatter(locale)

    fun format(rate: GemSwapRate): AssetRatePair = AssetRatePair(forward = format(rate.direct), reverse = format(rate.inverse))

    fun format(rate: GemAssetRate): String = "1 ${rate.baseSymbol} ≈ ${formatter.string(rate.value, rate.quoteSymbol)}"
}
