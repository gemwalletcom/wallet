package com.gemwallet.android.ui.components.swap

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.localization.text
import uniffi.gemstone.GemSlippageViewState

data class SlippageSuggestionUIModel(val label: String, val input: String)

data class SlippageStateUIModel(val isAuto: Boolean, val input: String, val placeholder: String, val footerText: String?, val suggestions: List<SlippageSuggestionUIModel>)

fun GemSlippageViewState.uiModel(context: Context): SlippageStateUIModel = SlippageStateUIModel(
    isAuto = isAuto,
    input = input,
    placeholder = placeholder,
    footerText = footer?.text(context),
    suggestions = suggestions.map { SlippageSuggestionUIModel(it.percent.text(), it.input) },
)
