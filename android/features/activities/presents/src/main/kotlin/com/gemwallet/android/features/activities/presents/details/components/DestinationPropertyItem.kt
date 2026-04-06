package com.gemwallet.android.features.activities.presents.details.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun DestinationPropertyItem(property: TransactionDetailsValue.Destination, listPosition: ListPosition) {
    when (property) {
        is TransactionDetailsValue.Destination.Recipient,
        is TransactionDetailsValue.Destination.Sender -> AddressPropertyItem(
            title = when (property) {
                is TransactionDetailsValue.Destination.Recipient -> R.string.transaction_recipient
                is TransactionDetailsValue.Destination.Sender -> R.string.transaction_sender
                else -> return
            },
            displayText = AddressFormatter(property.data).value(),
            copyValue = property.data,
            explorerLink = property.explorerLink,
            listPosition = listPosition,
        )
        is TransactionDetailsValue.Destination.Provider -> PropertyItem(
            title = { PropertyTitleText(R.string.common_provider) },
            data = { PropertyDataText(text = property.data) },
            listPosition = listPosition,
        )
    }
}
