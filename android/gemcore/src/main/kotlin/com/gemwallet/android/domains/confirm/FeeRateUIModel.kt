package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemFeeService
import java.math.BigInteger

private val feeService = GemFeeService()

data class FeeRateUIModel(
    val feeRate: GemFeeRate,
    val feeAsset: AssetInfo,
    val feeUnitType: FeeUnitType?,
    val feeRateDecimals: Int,
    val selectedRate: GemFeeRate? = null,
    val selectedFeeAmount: BigInteger? = null,
    val unitSymbol: String? = null,
) {
    val priority: FeePriority = FeePriority.entries.first { it.string == feeRate.priority }

    val price: String
        get() = if (feeUnitType == FeeUnitType.Native) {
            nativeAmountText()
        } else {
            gasPriceText()
        }

    val fiatValue: String
        get() = fiatText() ?: ""

    val emoji: String
        get() = when (priority) {
            FeePriority.Normal -> "\uD83D\uDC8E"
            FeePriority.Fast -> "\u26A1\uFE0F"
        }

    private val feeAmount: BigInteger?
        get() {
            if (selectedFeeAmount != null && selectedRate != null) {
                val selectedTotal = feeService.totalFee(selectedRate.gasPriceType).toBigInteger()
                if (selectedTotal == BigInteger.ZERO) return null
                return selectedFeeAmount.multiply(feeService.totalFee(feeRate.gasPriceType).toBigInteger()).divide(selectedTotal)
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
        feeUnitType ?: return ""
        val symbol = unitSymbol ?: return ""
        return ValueFormatter(style = ValueFormatter.Style.Auto)
            .string(feeService.totalFee(feeRate.gasPriceType).toBigInteger(), feeRateDecimals, symbol)
    }

    private fun nativeAmountText(): String {
        val amount = feeAmount ?: feeService.totalFee(feeRate.gasPriceType).toBigInteger()
        return ValueFormatter(style = ValueFormatter.Style.Auto)
            .string(amount, feeAsset.asset.decimals, feeAsset.asset.symbol)
    }
}
