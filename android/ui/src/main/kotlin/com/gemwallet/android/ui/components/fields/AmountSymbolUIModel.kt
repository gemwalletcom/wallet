package com.gemwallet.android.ui.components.fields

enum class AmountSymbolPlacement {
    Leading,
    Trailing,
}

data class AmountSymbolUIModel(val symbol: String, val placement: AmountSymbolPlacement)
