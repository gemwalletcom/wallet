package com.gemwallet.android.ui.models.swap

import com.wallet.core.primitives.swap.SwapPriceImpactType
import uniffi.gemstone.SwapperProvider

data class SwapRateUIModel(
    val forward: String,
    val reverse: String,
)

data class SwapProviderUIModel(
    val id: SwapperProvider,
    val title: String,
    val icon: String,
    val amount: String? = null,
    val fiat: String? = null,
)

data class SwapPriceImpactUIModel(
    val type: SwapPriceImpactType,
    val displayText: String,
    val warningText: String,
    val isHigh: Boolean,
)

data class SwapDetailsUIModel(
    val provider: SwapProviderUIModel,
    val providers: List<SwapProviderUIModel> = emptyList(),
    val rate: SwapRateUIModel,
    val priceImpact: SwapPriceImpactUIModel?,
    val minimumReceive: String,
    val slippageText: String,
    val estimatedTime: String? = null,
    val isProviderSelectable: Boolean = false,
) {
    val summaryPriceImpactText: String?
        get() = when (priceImpact?.type) {
            SwapPriceImpactType.Medium,
            SwapPriceImpactType.High -> priceImpact.displayText
            SwapPriceImpactType.Positive,
            SwapPriceImpactType.Low,
            null -> null
        }

    val summaryPriceImpactBadgeText: String?
        get() = summaryPriceImpactText?.let { "($it)" }

    val shouldShowPriceImpactWarning: Boolean
        get() = priceImpact?.isHigh == true
}
