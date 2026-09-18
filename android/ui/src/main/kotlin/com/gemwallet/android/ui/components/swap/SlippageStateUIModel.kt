package com.gemwallet.android.ui.components.swap

import android.content.Context
import com.gemwallet.android.ui.localization.footerText
import com.gemwallet.android.ui.models.swap.SwapSlippage
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
    footerText = if (isAuto) null else check.footerText(context, SwapSlippage.percentLabel(minimumBps, slippageText), SwapSlippage.percentLabel(maximumBps, slippageText)),
    suggestions = suggestionsBps.map { SlippageSuggestionUIModel(it, SwapSlippage.percentLabel(it, slippageText)) },
    maximumFractionDigits = maximumFractionDigits,
    maximumIntegerDigits = maximumIntegerDigits,
)
