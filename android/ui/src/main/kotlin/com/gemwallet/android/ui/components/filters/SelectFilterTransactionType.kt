package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.transactionFilters

fun LazyListScope.selectFilterTransactionType(
    filter: List<GemTransactionFilter>,
    onFilter: (GemTransactionFilter) -> Unit,
) {
    item {
        SubheaderItem(R.string.filter_types)
    }
    itemsPositioned(transactionFilters()) { position, item ->
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

fun GemTransactionFilter.getLabel() = when (this) {
    GemTransactionFilter.TRANSFERS -> R.string.transfer_title
    GemTransactionFilter.SWAPS -> R.string.wallet_swap
    GemTransactionFilter.STAKE -> R.string.wallet_stake
    GemTransactionFilter.SMART_CONTRACT -> R.string.transfer_smart_contract_title
    GemTransactionFilter.PERPETUALS -> R.string.perpetuals_title
    GemTransactionFilter.OTHERS -> R.string.transfer_other_title
}
