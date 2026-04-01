package com.gemwallet.android.ui.components.list_item.transaction

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.components.list_item.dateGroupedList

fun LazyListScope.transactionsList(
    items: List<TransactionDataAggregate>,
    onTransactionClick: (String) -> Unit
) {
    dateGroupedList(
        items = items,
        createdAt = { it.createdAt },
        key = { _, item -> item.id },
    ) { position, item ->
        TransactionItem(
            data = item,
            listPosition = position,
            onClick = { onTransactionClick(item.id) }
        )
    }
}
