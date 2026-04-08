package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
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

    when (model) {
        is ConfirmProperty.Destination.Transfer -> {
            val domain = model.domain
            if (domain != null) {
                PropertyItem(
                    title = { PropertyTitleText(R.string.transaction_recipient) },
                    data = {
                        Column(horizontalAlignment = Alignment.End) {
                            Row(horizontalArrangement = Arrangement.End) { PropertyDataText(model.displayData()) }
                        }
                    },
                    listPosition = listPosition,
                )
            } else {
                AddressPropertyItem(
                    title = R.string.transaction_recipient,
                    displayText = AddressFormatter(model.address).value(),
                    copyValue = model.address,
                    explorerLink = model.explorerLink,
                    listPosition = listPosition,
                )
            }
        }
        else -> {
            val title = when (model) {
                is ConfirmProperty.Destination.Provider -> R.string.common_provider
                is ConfirmProperty.Destination.Stake -> R.string.stake_validator
                is ConfirmProperty.Destination.Generic -> R.string.wallet_connect_app
                is ConfirmProperty.Destination.PerpetualOper -> R.string.common_provider
                is ConfirmProperty.Destination.Transfer -> return
            }
            PropertyItem(
                title = { PropertyTitleText(title) },
                data = {
                    Column(horizontalAlignment = Alignment.End) {
                        Row(horizontalArrangement = Arrangement.End) { PropertyDataText(model.displayData()) }
                    }
                },
                listPosition = listPosition,
            )
        }
    }
}

internal fun ConfirmProperty.Destination.displayData(): String = when (this) {
    is ConfirmProperty.Destination.Provider,
    is ConfirmProperty.Destination.Stake -> data
    is ConfirmProperty.Destination.Transfer -> domain ?: address
    is ConfirmProperty.Destination.Generic -> appName
    is ConfirmProperty.Destination.PerpetualOper -> providerName
}
