package com.gemwallet.android.domains.swap

import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Asset
import java.math.BigInteger

data class AssetRatePair(
    val forward: String,
    val reverse: String,
)

fun buildAssetRatePair(
    fromAsset: Asset,
    toAsset: Asset,
    fromValue: BigInteger,
    toValue: BigInteger,
    formatter: AssetRateFormatter = AssetRateFormatter(),
): AssetRatePair? {
    return try {
        val fromAmount = Crypto(fromValue).value(fromAsset.decimals)
        val toAmount = Crypto(toValue).value(toAsset.decimals)
        if (fromAmount.signum() == 0 || toAmount.signum() == 0) {
            return null
        }

        AssetRatePair(
            forward = formatter.format(
                fromAsset = fromAsset,
                toAsset = toAsset,
                fromAmount = fromAmount,
                toAmount = toAmount,
                direction = AssetRateFormatter.Direction.Direct,
            ),
            reverse = formatter.format(
                fromAsset = fromAsset,
                toAsset = toAsset,
                fromAmount = fromAmount,
                toAmount = toAmount,
                direction = AssetRateFormatter.Direction.Inverse,
            ),
        )
    } catch (_: Throwable) {
        null
    }
}
