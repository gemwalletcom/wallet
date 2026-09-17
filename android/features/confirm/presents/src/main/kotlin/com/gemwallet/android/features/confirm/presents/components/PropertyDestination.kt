package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmRowUIModel
import com.gemwallet.android.ui.LocalAddressService
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.localization.displayText
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Chain

@Composable
internal fun AddressRow(row: ConfirmRowUIModel.Address, listPosition: ListPosition) {
    AddressPropertyItem(
        title = row.title,
        displayText = destinationText(row.name, row.address, row.chain, row.avatar is ListItemImage.Stored),
        copyValue = row.address,
        image = row.avatar,
        explorerLink = row.explorerLink,
        listPosition = listPosition,
    )
}

@Composable
private fun destinationText(name: String?, address: String, chain: Chain?, hasImage: Boolean): String {
    val formatted = rememberFormattedAddress(address, chain)
    return LocalAddressService.current.displayText(name, formatted, hasImage)
}
