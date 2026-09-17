package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.domains.swap.AssetRatePair
import uniffi.gemstone.SwapPriceImpactType
import uniffi.gemstone.SwapProvider

data class SwapProviderUIModel(
    val id: SwapProvider,
    val title: String,
    val icon: Any?,
    val amount: String? = null,
    val fiat: String? = null,
)

data class SwapPriceImpactUIModel(
    val type: SwapPriceImpactType,
    val displayText: String,
    val warningText: String,
    val isHigh: Boolean,
    val showsInSummary: Boolean,
)

data class SwapDetailsUIModel(
    val rows: List<SwapDetailRowUIModel>,
    val provider: SwapProviderUIModel,
    val providers: List<SwapProviderUIModel> = emptyList(),
    val rate: AssetRatePair,
    val priceImpact: SwapPriceImpactUIModel?,
    val minimumReceive: String,
    val slippageText: String,
    val slippageBps: UInt,
    val selectedSlippage: UInt?,
    val etaInSeconds: UInt? = null,
    val isProviderSelectable: Boolean = false,
) {
    val summaryPriceImpactText: String?
        get() = priceImpact?.takeIf { it.showsInSummary }?.displayText

    val summaryPriceImpactBadgeText: String?
        get() = summaryPriceImpactText?.let { "($it)" }

    val shouldShowPriceImpactWarning: Boolean
        get() = priceImpact?.isHigh == true
}

sealed interface SwapDetailRowUIModel {
    data class Rate(val rate: AssetRatePair) : SwapDetailRowUIModel
    data class EstimatedTime(val seconds: UInt) : SwapDetailRowUIModel
    data class PriceImpact(val model: SwapPriceImpactUIModel) : SwapDetailRowUIModel
    data class MinimumReceive(val text: String) : SwapDetailRowUIModel
    data class Slippage(val text: String?) : SwapDetailRowUIModel
}
