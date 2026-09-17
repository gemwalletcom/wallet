package com.gemwallet.android.ui.components.list_item.transaction

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.components.list_item.DateSection
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.wallet.core.primitives.TransactionId

fun LazyListScope.transactionsList(
    sections: List<DateSection<TransactionDataAggregate>>,
    onTransactionClick: (TransactionId) -> Unit
) {
    dateSectionedList(
        sections = sections,
        key = { _, item -> item.id.identifier },
    ) { position, item ->
        TransactionItem(
            data = item,
            listPosition = position,
            onClick = { onTransactionClick(item.id) }
        )
    }
}
