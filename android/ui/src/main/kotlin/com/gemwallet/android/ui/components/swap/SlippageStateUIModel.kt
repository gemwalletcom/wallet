package com.gemwallet.android.ui.components.swap

import android.content.Context
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.swap.SwapSlippage
import uniffi.gemstone.GemSlippageCheck
import uniffi.gemstone.GemSlippageViewState

data class SlippageSuggestionUIModel(
    val bps: UInt,
    val label: String,
)

data class SlippageStateUIModel(
    val allowsConfirm: Boolean,
    val footerText: String?,
    val suggestions: List<SlippageSuggestionUIModel>,
    val maximumFractionDigits: UInt,
    val maximumIntegerDigits: UInt,
)

fun GemSlippageViewState.uiModel(context: Context, slippageText: (UInt) -> String): SlippageStateUIModel = SlippageStateUIModel(
    allowsConfirm = allowsConfirm,
    footerText = if (isAuto) null else when (check) {
        GemSlippageCheck.ABOVE_MAXIMUM -> context.getString(R.string.common_maximum_value, SwapSlippage.percentLabel(maximumBps, slippageText))
        GemSlippageCheck.BELOW_MINIMUM -> context.getString(R.string.common_minimum_value, SwapSlippage.percentLabel(minimumBps, slippageText))
        GemSlippageCheck.HIGH -> context.getString(R.string.swap_slippage_warning)
        GemSlippageCheck.VALID -> null
    },
    suggestions = suggestionsBps.map { SlippageSuggestionUIModel(it, SwapSlippage.percentLabel(it, slippageText)) },
    maximumFractionDigits = maximumFractionDigits,
    maximumIntegerDigits = maximumIntegerDigits,
)
