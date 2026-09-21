package com.gemwallet.android.ui.components.swap

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.localization.footerText
import uniffi.gemstone.GemSlippageViewState

data class SlippageSuggestionUIModel(val bps: UInt, val label: String)

data class SlippageStateUIModel(val allowsConfirm: Boolean, val footerText: String?, val suggestions: List<SlippageSuggestionUIModel>, val maximumFractionDigits: UInt, val maximumIntegerDigits: UInt)

fun GemSlippageViewState.uiModel(context: Context): SlippageStateUIModel = SlippageStateUIModel(
    allowsConfirm = allowsConfirm,
    footerText = if (isAuto) null else check.footerText(context, minimum.text(), maximum.text()),
    suggestions = suggestions.map { SlippageSuggestionUIModel(it.bps, it.percent.text()) },
    maximumFractionDigits = maximumFractionDigits,
    maximumIntegerDigits = maximumIntegerDigits,
)
