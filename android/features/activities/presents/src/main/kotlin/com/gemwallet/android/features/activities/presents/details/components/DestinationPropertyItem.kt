package com.gemwallet.android.features.activities.presents.details.components

import com.gemwallet.android.features.activities.presents.localization.stringRes
import com.gemwallet.android.ui.LocalAddressService
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.format.rememberFormattedAddress
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
        is TransactionDetailsValue.Destination.Sender,
        is TransactionDetailsValue.Destination.Contract,
        is TransactionDetailsValue.Destination.Validator,
        is TransactionDetailsValue.Destination.ProviderAddress -> AddressPropertyItem(
            title = property.stringRes(),
            displayText = property.name ?: rememberFormattedAddress(property.data, property.chain),
            copyValue = property.data,
            explorerLink = property.explorerLink,
            listPosition = listPosition,
        )
        is TransactionDetailsValue.Destination.Provider -> PropertyItem(
            title = { PropertyTitleText(property.stringRes()) },
            data = { PropertyDataText(text = property.data) },
            listPosition = listPosition,
        )
    }
}
