package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.fiat_connect.viewmodels.models.FiatTransactionRowUIModel
import com.gemwallet.android.ui.components.list_item.DateSection
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.wallet.core.primitives.FiatTransactionAssetData

fun LazyListScope.fiatTransactionsList(sections: List<DateSection<FiatTransactionRowUIModel>>, onTransactionClick: (FiatTransactionAssetData) -> Unit) {
    dateSectionedList(
        sections = sections,
        key = { _, item -> item.data.id },
    ) { position, item ->
        ListItem(
            model = item.model,
            listPosition = position,
            modifier = Modifier.clickable { onTransactionClick(item.data) },
        )
    }
}
