package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemCustomFee
import java.math.BigInteger

data class CustomFee(
    val rate: BigInteger?,
    val placeholder: String,
    val networkFee: FeeUIModel.FeeInfo,
    val maxRateText: String,
    val minRateText: String,
    val isOverMax: Boolean,
    val isBelowMinimum: Boolean,
    val isConfirmEnabled: Boolean,
) {
    companion object {
        fun from(
            input: String,
            currentFee: FeeUIModel.FeeInfo,
            feeRates: List<GemFeeRate>,
            selection: FeeSelection,
            decimals: Int,
        ): CustomFee {
            val baseTotal = baseTotal(selection, feeRates, currentFee.priority)
            val normalTotal = normalTotal(feeRates) ?: baseTotal
            val rate = input.parseInputNumberOrNull()?.movePointRight(decimals)?.toBigInteger()?.takeIf { it > BigInteger.ZERO }

            return GemCustomFee.estimate(
                chain = currentFee.feeAsset.chain.string,
                rate = rate,
                loadedFee = currentFee.amount,
                baseTotal = baseTotal,
                normalTotal = normalTotal,
            ).use { estimate ->
                CustomFee(
                    rate = rate,
                    placeholder = ValueFormatter(style = ValueFormatter.Style.Auto).string(baseTotal, decimals),
                    networkFee = FeeUIModel.FeeInfo(estimate.feeValue(), currentFee.feeAsset, currentFee.price, currentFee.currency, currentFee.priority),
                    maxRateText = format(estimate.maxRate(), decimals),
                    minRateText = estimate.minimumRate()?.let { format(it, decimals) } ?: "",
                    isOverMax = estimate.isOverMax(),
                    isBelowMinimum = estimate.isBelowMinimum(),
                    isConfirmEnabled = estimate.isValid(),
                )
            }
        }

        fun format(value: BigInteger, decimals: Int): String =
            value.toBigDecimal().movePointLeft(decimals).stripTrailingZeros().toPlainString()

        fun formatRate(value: BigInteger, decimals: Int, unitSymbol: String): String =
            ValueFormatter(style = ValueFormatter.Style.Auto).string(value, decimals, unitSymbol)

        private fun baseTotal(selection: FeeSelection, feeRates: List<GemFeeRate>, loadedPriority: FeePriority): BigInteger =
            when (selection) {
                is FeeSelection.Custom -> selection.gasPrice
                is FeeSelection.Preset -> feeRates.firstOrNull { it.priority.toPrimitives() == loadedPriority }
                    ?.let { it.gasPriceType.totalFee() } ?: BigInteger.ZERO
            }

        private fun normalTotal(feeRates: List<GemFeeRate>): BigInteger? =
            (feeRates.firstOrNull { it.priority.toPrimitives() == FeePriority.Normal } ?: feeRates.firstOrNull())
                ?.let { it.gasPriceType.totalFee() }
    }
}
