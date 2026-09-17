package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.ListPosition

fun LazyListScope.confirmBalanceChangesContent(rows: List<ListItemModel>) {
    itemsIndexed(rows) { index, row ->
        ListItem(model = row, listPosition = ListPosition.getPosition(index, rows.size))
    }
}
