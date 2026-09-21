package com.gemwallet.android.ui.components.fields

import androidx.annotation.StringRes
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle

sealed interface NameResolveIndicatorUIModel {
    data object Loading : NameResolveIndicatorUIModel
    data class Icon(val symbol: ListItemSymbol, val style: ListItemTextStyle, @param:StringRes val contentDescription: Int?) : NameResolveIndicatorUIModel
}
