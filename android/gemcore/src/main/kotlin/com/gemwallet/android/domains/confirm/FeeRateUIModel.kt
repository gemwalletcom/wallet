package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.SignMode
import com.gemwallet.android.model.format
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
            FeePriority.Slow -> "\uD83D\uDC22"
            FeePriority.Normal -> "\uD83D\uDC8E"
            FeePriority.Fast -> "\uD83D\uDE80"
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
        val fiat = Crypto(amount).convert(feeAsset.asset.decimals, priceInfo.price.price)
        return priceInfo.currency.format(fiat.atomicValue, dynamicPlace = true)
    }

    private fun gasPriceText(): String {
        val unit = feeUnitType ?: return ""
        val decimals = unit.gasPriceDecimals ?: return ""
        val symbol = unit.gasPriceSymbol ?: return ""
        return Crypto(feeRate.gasPriceType.totalFee()).format(decimals, symbol, 2, -1, SignMode.NoSign, true)
    }
}
