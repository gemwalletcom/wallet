package com.gemwallet.android.features.wallet.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun WalletAddress(
    addresses: List<String>,
) {
    // Show if single account wallet
    val address = addresses.takeIf { it.size == 1 }?.firstOrNull() ?: return

    AddressPropertyItem(
        title = R.string.common_address,
        displayText = AddressFormatter(address).value(),
        copyValue = address,
        listPosition = ListPosition.Single,
    )
}
