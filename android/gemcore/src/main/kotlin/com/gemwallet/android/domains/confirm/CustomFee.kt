package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toFeePriority
import com.gemwallet.android.ext.totalFee
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.customFeeEstimate
import java.math.BigInteger

data class CustomFee(
    val rate: BigInteger?,
    val placeholder: String,
    val networkFee: FeeUIModel.FeeInfo,
    val maxRateText: String,
    val isOverMax: Boolean,
    val isConfirmEnabled: Boolean,
) {
    companion object {
        fun from(
            input: String,
            currentFee: FeeUIModel.FeeInfo,
            feeRates: List<GemFeeRate>,
            selection: FeeSelection,
            decimals: Int,
            maxMultiplier: Int,
        ): CustomFee {
            val baseTotal = baseTotal(selection, feeRates, currentFee.priority)
            val normalTotal = normalTotal(feeRates) ?: baseTotal
            val rate = input.parseInputNumberOrNull()?.movePointRight(decimals)?.toBigInteger()?.takeIf { it > BigInteger.ZERO }

            val estimate = customFeeEstimate(
                rate = rate?.toString(),
                loadedFee = currentFee.amount.toString(),
                baseTotal = baseTotal.toString(),
                normalTotal = normalTotal.toString(),
                maxMultiplier = maxMultiplier.toUInt(),
            )

            return CustomFee(
                rate = rate,
                placeholder = ValueFormatter(style = ValueFormatter.Style.Auto).string(baseTotal, decimals),
                networkFee = FeeUIModel.FeeInfo(BigInteger(estimate.feeValue), currentFee.feeAsset, currentFee.price, currentFee.currency, currentFee.priority),
                maxRateText = format(BigInteger(estimate.maxRate), decimals),
                isOverMax = estimate.isOverMax,
                isConfirmEnabled = rate != null && !estimate.isOverMax,
            )
        }

        fun format(value: BigInteger, decimals: Int): String =
            value.toBigDecimal().movePointLeft(decimals).stripTrailingZeros().toPlainString()

        fun formatRate(value: BigInteger, decimals: Int, unitSymbol: String): String =
            ValueFormatter(style = ValueFormatter.Style.Auto).string(value, decimals, unitSymbol)

        private fun baseTotal(selection: FeeSelection, feeRates: List<GemFeeRate>, loadedPriority: FeePriority): BigInteger =
            when (selection) {
                is FeeSelection.Custom -> selection.gasPrice
                is FeeSelection.Preset -> feeRates.firstOrNull { it.priority.toFeePriority() == loadedPriority }
                    ?.gasPriceType?.totalFee() ?: BigInteger.ZERO
            }

        private fun normalTotal(feeRates: List<GemFeeRate>): BigInteger? =
            (feeRates.firstOrNull { it.priority.toFeePriority() == FeePriority.Normal } ?: feeRates.firstOrNull())
                ?.gasPriceType?.totalFee()
    }
}
