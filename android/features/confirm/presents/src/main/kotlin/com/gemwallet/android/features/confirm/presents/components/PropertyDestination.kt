package com.gemwallet.android.features.confirm.presents.components

import com.gemwallet.android.ui.LocalAddressService
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.R
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.features.confirm.presents.localization.title
import com.wallet.core.primitives.AddressType
import uniffi.gemstone.contactInitials
import uniffi.gemstone.GemAddressDisplay
import com.wallet.core.primitives.Chain

@Composable
fun PropertyDestination(
    model: ConfirmProperty.Destination?,
    listPosition: ListPosition,
) {
    model ?: return

    when (model) {
        is ConfirmProperty.Destination.Transfer -> {
            val (icon, initials) = when (model.addressType) {
                AddressType.Contact -> walletImageModel(LocalContext.current, model.imageUrl) to model.domain?.let { contactInitials(it) }
                else -> null to null
            }
            AddressPropertyItem(
                title = model.titleRes(),
                displayText = destinationText(model.domain, model.address, model.chain, icon != null),
                copyValue = model.address,
                icon = icon,
                placeholderText = initials,
                explorerLink = model.explorerLink,
                listPosition = listPosition,
            )
        }
        is ConfirmProperty.Destination.Contract -> AddressPropertyItem(
            title = model.titleRes(),
            displayText = rememberFormattedAddress(model.address, model.chain),
            copyValue = model.address,
            explorerLink = model.explorerLink,
            listPosition = listPosition,
        )
        is ConfirmProperty.Destination.Resource -> PropertyItem(
            title = { PropertyTitleText(model.titleRes()) },
            data = { PropertyDataText(stringResource(model.resource.stringRes())) },
            listPosition = listPosition,
        )
        is ConfirmProperty.Destination.Stake -> {
            val address = model.address
            if (address != null && model.explorerLink != null) {
                AddressPropertyItem(
                    title = model.titleRes(),
                    displayText = model.displayData(),
                    copyValue = address,
                    explorerLink = model.explorerLink,
                    listPosition = listPosition,
                )
            } else {
                PropertyItem(
                    title = { PropertyTitleText(model.titleRes()) },
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
            PropertyItem(
                title = { PropertyTitleText(model.titleRes()) },
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

internal fun ConfirmProperty.Destination.titleRes(): Int = kind?.title() ?: R.string.wallet_connect_app

internal fun ConfirmProperty.Destination.displayData(): String = when (this) {
    is ConfirmProperty.Destination.Provider,
    is ConfirmProperty.Destination.Stake,
    is ConfirmProperty.Destination.Resource -> data
    is ConfirmProperty.Destination.Contract -> address
    is ConfirmProperty.Destination.Transfer -> domain ?: address
    is ConfirmProperty.Destination.Generic -> appName
}

@Composable
private fun destinationText(name: String?, address: String, chain: Chain?, hasImage: Boolean): String {
    val formatted = rememberFormattedAddress(address, chain)
    return when (val display = LocalAddressService.current.display(name, formatted, hasImage)) {
        is GemAddressDisplay.Address -> formatted
        is GemAddressDisplay.Name -> display.name
        is GemAddressDisplay.NameWithAddress -> "${display.name} ($formatted)"
    }
}
