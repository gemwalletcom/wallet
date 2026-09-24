package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.text
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRateRow
import uniffi.gemstone.feeAmount

data class FeeRateUIModel(val row: GemFeeRateRow, val feeAsset: AssetPriceValue) {
    val priority: FeePriority = row.priority.toPrimitives()

    val fiatValue: String
        get() {
            val priceInfo = feeAsset.price ?: return ""
            val fee = row.fee ?: return ""
            return feeAmount(feeAsset.asset.toGem(), fee, priceInfo.price.price, priceInfo.currency.toGem()).fiat?.text().orEmpty()
        }

    val emoji: String
        get() = when (priority) {
            FeePriority.Normal -> "\uD83D\uDC8E"
            FeePriority.Fast -> "\u26A1\uFE0F"
        }
}
