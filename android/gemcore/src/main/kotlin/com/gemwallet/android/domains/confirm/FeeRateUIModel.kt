package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.FeePriority
import com.gemwallet.android.ext.gasPriceDecimals
import com.gemwallet.android.ext.gasPriceSymbol
import com.gemwallet.android.ext.totalFee
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemFeeRate
import java.math.BigInteger

data class FeeRateUIModel(
    val feeRate: GemFeeRate,
    val feeAsset: AssetInfo,
    val feeUnitType: FeeUnitType?,
    val selectedRate: GemFeeRate? = null,
    val selectedFeeAmount: BigInteger? = null,
) {
    val priority: FeePriority = FeePriority.entries.first { it.string == feeRate.priority }

    val price: String
        get() = if (feeUnitType == FeeUnitType.Native) {
            fiatText() ?: ""
        } else {
            gasPriceText()
        }

    val fiatValue: String
        get() = if (feeUnitType == FeeUnitType.Native) "" else (fiatText() ?: "")

    val emoji: String
        get() = when (priority) {
            FeePriority.Slow -> "\u23F1\uFE0F"
            FeePriority.Normal -> "\uD83D\uDC8E"
            FeePriority.Fast -> "\u26A1\uFE0F"
        }

    private val feeAmount: BigInteger?
        get() {
            if (selectedFeeAmount != null && selectedRate != null) {
                val selectedTotal = selectedRate.gasPriceType.totalFee()
                if (selectedTotal == BigInteger.ZERO) return null
                return selectedFeeAmount.multiply(feeRate.gasPriceType.totalFee()).divide(selectedTotal)
            }
            return null
        }

    private fun fiatText(): String? {
        val priceInfo = feeAsset.price ?: return null
        val amount = feeAmount ?: return null
        val fiat = CryptoFiatConverter.toFiat(Crypto(amount), feeAsset.asset.decimals, priceInfo.price.price)
        return CurrencyFormatter(currency = priceInfo.currency).string(fiat.atomicValue)
    }

    private fun gasPriceText(): String {
        val unit = feeUnitType ?: return ""
        val decimals = unit.gasPriceDecimals ?: return ""
        val symbol = unit.gasPriceSymbol ?: return ""
        return ValueFormatter(style = ValueFormatter.Style.Auto)
            .string(feeRate.gasPriceType.totalFee(), decimals, symbol)
    }
}
