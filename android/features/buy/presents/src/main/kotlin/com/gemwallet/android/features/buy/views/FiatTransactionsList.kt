package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.components.list_item.dateGroupedList
import com.wallet.core.primitives.FiatTransactionAssetData

fun LazyListScope.fiatTransactionsList(
    items: List<FiatTransactionAssetData>,
    onTransactionClick: (FiatTransactionAssetData) -> Unit,
) {
    dateGroupedList(
        items = items,
        createdAt = { it.transaction.createdAt },
        key = { _, item -> item.transaction.id },
    ) { position, item ->
        FiatTransactionItem(
            info = item,
            listPosition = position,
            onClick = { onTransactionClick(item) }
        )
    }
}
