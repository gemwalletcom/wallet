package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.MerchantPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.wallet.core.primitives.AddressType

@Composable
fun PropertyDestination(
    model: ConfirmProperty.Destination?,
    listPosition: ListPosition,
) {
    model ?: return

    when (model) {
        is ConfirmProperty.Destination.Transfer -> {
            val (icon, initials) = when (model.addressType) {
                AddressType.Contact -> walletImageModel(LocalContext.current, model.imageUrl) to model.domain?.take(2)?.uppercase()
                else -> null to null
            }
            AddressPropertyItem(
                title = R.string.transaction_recipient,
                displayText = model.domain ?: AddressFormatter(model.address, chain = model.chain).value(),
                copyValue = model.address,
                icon = icon,
                placeholderText = initials,
                explorerLink = model.explorerLink,
                listPosition = listPosition,
            )
        }
        is ConfirmProperty.Destination.Merchant -> MerchantPropertyItem(
            title = R.string.transfer_recipient_title,
            name = model.displayData(),
            iconUrl = model.iconUrl,
            listPosition = listPosition,
        )
        is ConfirmProperty.Destination.Stake -> {
            val address = model.address
            if (address != null && model.explorerLink != null) {
                AddressPropertyItem(
                    title = R.string.stake_validator,
                    displayText = model.displayData(),
                    copyValue = address,
                    explorerLink = model.explorerLink,
                    listPosition = listPosition,
                )
            } else {
                PropertyItem(
                    title = { PropertyTitleText(R.string.stake_validator) },
                    data = {
                        Column(horizontalAlignment = Alignment.End) {
                            Row(horizontalArrangement = Arrangement.End) { PropertyDataText(model.displayData()) }
                        }
                    },
                    listPosition = listPosition,
                )
            }
        }
        else -> {
            val title = when (model) {
                is ConfirmProperty.Destination.Provider -> R.string.common_provider
                is ConfirmProperty.Destination.Generic -> R.string.wallet_connect_app
                is ConfirmProperty.Destination.PerpetualOper -> R.string.common_provider
                is ConfirmProperty.Destination.Merchant,
                is ConfirmProperty.Destination.Stake,
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
    is ConfirmProperty.Destination.Merchant,
    is ConfirmProperty.Destination.Stake -> data
    is ConfirmProperty.Destination.Transfer -> domain ?: address
    is ConfirmProperty.Destination.Generic -> appName
    is ConfirmProperty.Destination.PerpetualOper -> providerName
}
