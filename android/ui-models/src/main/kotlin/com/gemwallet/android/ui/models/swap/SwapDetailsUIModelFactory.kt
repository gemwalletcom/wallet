package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemSwapProviderRow
import uniffi.gemstone.GemSwapQuoteSummary
import uniffi.gemstone.GemValueStyle
import java.math.BigInteger

data class SwapDetailsUIModelInput(
    val payAsset: AssetPriceValue,
    val receiveAsset: AssetPriceValue,
    val summary: GemSwapQuoteSummary,
    val provider: GemSwapProviderRow,
    val providers: List<GemSwapProviderRow> = emptyList(),
    val slippageBps: UInt,
    val selectedSlippage: UInt?,
    val isProviderSelectable: Boolean,
)

object SwapDetailsUIModelFactory {
    private val rateFormatter = AssetRateFormatter()

    fun create(input: SwapDetailsUIModelInput): SwapDetailsUIModel? {
        val rate = input.summary.rate?.let(rateFormatter::format) ?: return null

        val slippagePercent = input.summary.slippagePercent()

        val minReceiveAtomic = input.summary.minReceiveValue
        val minimumReceive = ValueFormatter(style = GemValueStyle.AUTO).string(minReceiveAtomic, input.receiveAsset.asset)
        val slippageText = slippagePercent.formatAsPercentage(style = GemPercentageStyle.UNSIGNED)
        val rows = input.summary.detailRows(input.selectedSlippage != null)

        return SwapDetailsUIModel(
            rows = rows,
            provider = input.provider,
            providers = input.providers,
            rate = rate,
            priceImpact = input.summary.priceImpactRow,
            minimumReceive = minimumReceive,
            slippageText = slippageText,
            slippageBps = input.slippageBps,
            selectedSlippage = input.selectedSlippage,
            etaInSeconds = input.summary.quote.etaInSeconds,
            isProviderSelectable = input.isProviderSelectable,
        )
    }
}
