package com.gemwallet.android.ui.components.list_item

sealed interface SwapProgressMarkerUIModel {
    data object Spinner : SwapProgressMarkerUIModel
    data object Dots : SwapProgressMarkerUIModel
    data class Icon(val symbol: ListItemSymbol) : SwapProgressMarkerUIModel
}
