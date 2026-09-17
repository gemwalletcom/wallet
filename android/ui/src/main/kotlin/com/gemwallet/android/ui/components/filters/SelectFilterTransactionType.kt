package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned

fun LazyListScope.selectFilterTransactionType(
    filters: List<TransactionFilterUIModel>,
    filter: List<TransactionFilterUIModel>,
    onFilter: (TransactionFilterUIModel) -> Unit,
) {
    item {
        SubheaderItem(R.string.filter_types)
    }
    itemsPositioned(filters) { position, item ->
        ListItem(
            model = ListItemModel(title = item.title),
            listPosition = position,
            modifier = Modifier.clickable { onFilter(item) },
            accessory = if (filter.contains(item)) {
                { SelectionCheckmark() }
            } else {
                null
            },
        )
    }
}
