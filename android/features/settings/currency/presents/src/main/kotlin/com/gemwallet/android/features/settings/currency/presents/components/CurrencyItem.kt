package com.gemwallet.android.features.settings.currency.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemCurrencyRow

@Composable
fun CurrencyItem(
    row: GemCurrencyRow,
    isSelected: Boolean,
    listPosition: ListPosition,
    onSelect: (GemCurrencyRow) -> Unit,
) {
    val title = android.icu.util.Currency.getInstance(row.currency).displayName

    ListItem(
        modifier = Modifier.clickable { onSelect(row) },
        minHeight = ListItemDefaults.plainMinHeight,
        title = { ListItemTitleText("${row.flag}  ${row.currency} - $title") },
        listPosition = listPosition,
        trailing = if (isSelected) {
            @Composable { SelectionCheckmark() }
        } else {
            null
        },
    )
}
