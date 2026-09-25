package com.gemwallet.android.ui.components.list_item.transaction

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.components.list_item.DateSection
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.wallet.core.primitives.TransactionId
import uniffi.gemstone.GemTransactionRow

fun LazyListScope.transactionsList(sections: List<DateSection<GemTransactionRow>>, onTransactionClick: (TransactionId) -> Unit) {
    dateSectionedList(
        sections = sections,
        key = { _, item -> item.id },
    ) { position, item ->
        TransactionItem(
            data = item,
            listPosition = position,
            onClick = { onTransactionClick(TransactionId(item.id)) },
        )
    }
}
