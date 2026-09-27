package com.gemwallet.android.features.transfer.presents.confirm.components

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemListRow

fun LazyListScope.confirmBalanceChangesContent(rows: List<GemListRow>) {
    itemsIndexed(rows) { index, row ->
        GemListRowView(row = row, listPosition = ListPosition.getPosition(index, rows.size))
    }
}
