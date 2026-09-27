package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.ui.components.list_item.DateSection
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.gemwallet.android.ui.components.list_item.listItemModel
import uniffi.gemstone.GemFiatTransactionRow

fun LazyListScope.fiatTransactionsList(sections: List<DateSection<GemFiatTransactionRow>>, onTransactionClick: (GemFiatTransactionRow) -> Unit) {
    dateSectionedList(
        sections = sections,
        key = { _, row -> row.id },
    ) { position, row ->
        ListItem(
            model = row.listItemModel(LocalContext.current),
            listPosition = position,
            modifier = Modifier.clickable { onTransactionClick(row) },
        )
    }
}
