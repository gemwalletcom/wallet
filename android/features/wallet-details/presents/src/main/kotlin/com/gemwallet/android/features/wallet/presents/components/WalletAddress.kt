package com.gemwallet.android.features.wallet.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.LocalAddressService
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun WalletAddress(account: ChainAddress?, explorerLink: BlockExplorerLink?) {
    account ?: return

    AddressPropertyItem(
        title = R.string.common_address,
        displayText = rememberFormattedAddress(account.address, account.chain),
        copyValue = account.address,
        explorerLink = explorerLink,
        listPosition = ListPosition.Single,
    )
}
