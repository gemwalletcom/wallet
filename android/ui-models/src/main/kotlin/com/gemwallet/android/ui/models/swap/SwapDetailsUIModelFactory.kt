package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.swap.AssetRateFormatter
import uniffi.gemstone.GemSwapProviderRow
import uniffi.gemstone.GemSwapQuoteSummary

data class SwapDetailsUIModelInput(
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

        val rows = input.summary.detailRows(input.selectedSlippage != null)

        return SwapDetailsUIModel(
            rows = rows,
            provider = input.provider,
            providers = input.providers,
            rate = rate,
            priceImpact = input.summary.priceImpactRow,
            slippageBps = input.slippageBps,
            selectedSlippage = input.selectedSlippage,
            isProviderSelectable = input.isProviderSelectable,
        )
    }
}
