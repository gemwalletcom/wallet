package com.gemwallet.android.features.wallets.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemAddressRow

@Composable
internal fun WalletAddress(row: GemAddressRow?) {
    row ?: return

    AddressPropertyItem(row = row, listPosition = ListPosition.Single)
}
