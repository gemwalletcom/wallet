package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.model.text
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemSwapDetails
import uniffi.gemstone.GemSwapPriceImpactRow
import uniffi.gemstone.GemSwapProviderRow

data class SwapDetailsUIModel(
    val rows: List<GemListRow>,
    val provider: GemSwapProviderRow,
    val providers: List<GemSwapProviderRow> = emptyList(),
    val rate: AssetRatePair,
    val priceImpact: GemSwapPriceImpactRow?,
    val isProviderSelectable: Boolean = false,
) {
    val summaryPriceImpactText: String?
        get() = priceImpact?.takeIf { it.showsInSummary }?.value?.text()

    val summaryPriceImpactBadgeText: String?
        get() = summaryPriceImpactText?.let { "($it)" }

    val shouldShowPriceImpactWarning: Boolean
        get() = priceImpact?.warning != null
}

fun GemSwapDetails.uiModel(providers: List<GemSwapProviderRow> = emptyList(), isProviderSelectable: Boolean = false): SwapDetailsUIModel? {
    val rate = summary.rate?.let(AssetRateFormatter()::format) ?: return null
    return SwapDetailsUIModel(
        rows = rows,
        provider = provider,
        providers = providers,
        rate = rate,
        priceImpact = summary.priceImpactRow,
        isProviderSelectable = isProviderSelectable,
    )
}
