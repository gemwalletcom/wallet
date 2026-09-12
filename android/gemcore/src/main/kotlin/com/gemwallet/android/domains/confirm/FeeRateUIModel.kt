package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRateRow

data class FeeRateUIModel(
    val row: GemFeeRateRow,
    val feeAsset: AssetPriceValue,
    val feeRateDecimals: Int,
    val unitSymbol: String,
) {
    val priority: FeePriority = row.priority.toPrimitives()

    val price: String
        get() = ValueFormatter(style = ValueFormatter.Style.Auto)
            .string(row.displayValue, feeRateDecimals, unitSymbol)

    val fiatValue: String
        get() {
            val priceInfo = feeAsset.price ?: return ""
            val fee = row.fee ?: return ""
            val fiat = CryptoFiatConverter.toFiat(Crypto(fee), feeAsset.asset.decimals, priceInfo.price.price)
            return CurrencyFormatter(currency = priceInfo.currency).string(fiat.atomicValue)
        }

    val emoji: String
        get() = when (priority) {
            FeePriority.Normal -> "\uD83D\uDC8E"
            FeePriority.Fast -> "\u26A1\uFE0F"
        }
}
