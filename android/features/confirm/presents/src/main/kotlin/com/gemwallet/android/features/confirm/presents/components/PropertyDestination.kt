package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.confirm.models.ConfirmProperty

@Composable
fun PropertyDestination(
    model: ConfirmProperty.Destination?,
    listPosition: ListPosition,
) {
    model ?: return

    val title = when (model) {
        is ConfirmProperty.Destination.Provider -> R.string.common_provider
        is ConfirmProperty.Destination.Stake -> R.string.stake_validator
        is ConfirmProperty.Destination.Transfer -> R.string.transaction_recipient
        is ConfirmProperty.Destination.Generic -> R.string.wallet_connect_app
        is ConfirmProperty.Destination.PerpetualOper -> R.string.common_provider
    }
    PropertyItem(
        title = {
            PropertyTitleText(title)
        },
        data = {
            Column(horizontalAlignment = Alignment.End) {
                Row(horizontalArrangement = Arrangement.End) { PropertyDataText(model.displayData()) }
            }
        },
        listPosition = listPosition,
    )
}

internal fun ConfirmProperty.Destination.displayData(): String {
    return when (this) {
        is ConfirmProperty.Destination.Stake,
        is ConfirmProperty.Destination.Provider -> data
        is ConfirmProperty.Destination.Transfer -> domain ?: AddressFormatter(address).value()
        is ConfirmProperty.Destination.Generic -> appName
        is ConfirmProperty.Destination.PerpetualOper -> providerName
    }
}
