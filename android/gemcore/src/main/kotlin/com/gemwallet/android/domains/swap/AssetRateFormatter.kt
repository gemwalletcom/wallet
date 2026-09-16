package com.gemwallet.android.domains.swap

import com.gemwallet.android.model.text
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemSwapRate
import java.util.Locale

data class AssetRatePair(
    val forward: String,
    val reverse: String,
)

class AssetRateFormatter(
    private val locale: Locale = Locale.getDefault(),
) {

    fun format(rate: GemSwapRate): AssetRatePair = AssetRatePair(forward = format(rate.direct), reverse = format(rate.inverse))

    fun format(rate: GemAssetRate): String = rate.text(rate.value.text(locale))
}
