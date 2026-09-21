package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmRowUIModel
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun AddressRow(row: ConfirmRowUIModel.Address, listPosition: ListPosition, onClick: () -> Unit) {
    AddressPropertyItem(
        title = row.title,
        displayText = row.name ?: rememberFormattedAddress(row.address, row.chain),
        copyValue = row.address,
        image = row.avatar,
        explorerLink = row.explorerLink,
        listPosition = listPosition,
        onClick = onClick,
    )
}
