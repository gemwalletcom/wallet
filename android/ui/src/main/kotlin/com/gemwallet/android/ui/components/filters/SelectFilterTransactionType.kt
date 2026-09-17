package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.localization.getLabel
import uniffi.gemstone.GemTransactionFilter

fun LazyListScope.selectFilterTransactionType(
    filters: List<GemTransactionFilter>,
    filter: List<GemTransactionFilter>,
    onFilter: (GemTransactionFilter) -> Unit,
) {
    item {
        SubheaderItem(R.string.filter_types)
    }
    itemsPositioned(filters) { position, item ->
        ListItem(
            model = ListItemModel(title = stringResource(item.getLabel())),
            listPosition = position,
            modifier = Modifier.clickable { onFilter(item) },
            accessory = if (filter.contains(item)) {
                { SelectionCheckmark() }
            } else {
                null
            },
        )
    }
}
