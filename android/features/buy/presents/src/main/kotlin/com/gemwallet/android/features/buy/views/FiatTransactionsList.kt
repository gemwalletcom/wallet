package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.components.list_item.DateSection
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.wallet.core.primitives.FiatTransactionAssetData

fun LazyListScope.fiatTransactionsList(
    sections: List<DateSection<FiatTransactionAssetData>>,
    onTransactionClick: (FiatTransactionAssetData) -> Unit,
) {
    dateSectionedList(
        sections = sections,
        key = { _, item -> item.id },
    ) { position, item ->
        FiatTransactionItem(
            info = item,
            listPosition = position,
            onClick = { onTransactionClick(item) }
        )
    }
}
