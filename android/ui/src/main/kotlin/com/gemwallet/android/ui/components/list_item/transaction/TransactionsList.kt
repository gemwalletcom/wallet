package com.gemwallet.android.ui.components.list_item.transaction

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.components.list_item.dateGroupedList
import com.wallet.core.primitives.TransactionId

fun LazyListScope.transactionsList(
    items: List<TransactionDataAggregate>,
    onTransactionClick: (TransactionId) -> Unit
) {
    dateGroupedList(
        items = items,
        createdAt = { it.createdAt },
        key = { _, item -> item.id.identifier },
    ) { position, item ->
        TransactionItem(
            data = item,
            listPosition = position,
            onClick = { onTransactionClick(item.id) }
        )
    }
}
