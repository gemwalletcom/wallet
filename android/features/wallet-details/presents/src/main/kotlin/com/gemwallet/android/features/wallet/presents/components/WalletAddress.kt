package com.gemwallet.android.features.wallet.presents.components

import com.gemwallet.android.ui.LocalAddressService
import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun WalletAddress(
    accounts: List<ChainAddress>,
) {
    val account = accounts.takeIf { it.size == 1 }?.firstOrNull() ?: return

    AddressPropertyItem(
        title = R.string.common_address,
        displayText = AddressFormatter(LocalAddressService.current, address = account.address, chain = account.chain).value(),
        copyValue = account.address,
        listPosition = ListPosition.Single,
    )
}
