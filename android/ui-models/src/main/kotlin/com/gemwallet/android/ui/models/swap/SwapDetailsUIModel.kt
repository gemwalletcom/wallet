package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.model.text
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemSwapPriceImpactRow
import uniffi.gemstone.GemSwapProviderRow
import uniffi.gemstone.SwapProvider

data class SwapDetailsUIModel(
    val rows: List<GemListRow>,
    val provider: GemSwapProviderRow,
    val providers: List<GemSwapProviderRow> = emptyList(),
    val rate: AssetRatePair,
    val priceImpact: GemSwapPriceImpactRow?,
    val slippageBps: UInt,
    val selectedSlippage: UInt?,
    val etaInSeconds: UInt? = null,
    val isProviderSelectable: Boolean = false,
) {
    val summaryPriceImpactText: String?
        get() = priceImpact?.takeIf { it.showsInSummary }?.value?.text()

    val summaryPriceImpactBadgeText: String?
        get() = summaryPriceImpactText?.let { "($it)" }

    val shouldShowPriceImpactWarning: Boolean
        get() = priceImpact?.warning != null
}
