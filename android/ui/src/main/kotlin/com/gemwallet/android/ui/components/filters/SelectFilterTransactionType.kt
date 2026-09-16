package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.getLabel
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
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
        PropertyItem(
            modifier = Modifier.clickable { onFilter(item) },
            title = { PropertyTitleText(item.getLabel()) },
            data = {
                if (filter.contains(item)) {
                    SelectionCheckmark()
                }
            },
            listPosition = position,
        )
    }
}
