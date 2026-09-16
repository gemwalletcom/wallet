package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemRecipientSection
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.format.rememberFormattedAddresses
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType

@Composable
fun rememberWalletAddresses(wallets: List<Wallet>, toChain: Chain): Map<String, String> = rememberFormattedAddresses(
    remember(wallets, toChain) {
        wallets.mapNotNull { wallet -> wallet.getAccount(toChain)?.let { ChainAddress(chain = it.chain, address = it.address) } }
    }
)

fun LazyListScope.walletsSection(
    section: GemRecipientSection,
    wallets: List<Wallet>,
    toChain: Chain,
    addresses: Map<String, String>,
    onSelect: (Wallet, Account) -> Unit,
) {
    val entries = wallets.mapNotNull { wallet -> wallet.getAccount(toChain)?.let { wallet to it } }
    if (entries.isEmpty()) {
        return
    }
    item {
        SubheaderItem(section.stringRes())
    }
    itemsIndexed(entries) { index, (wallet, account) ->
        WalletRecipient(wallet, account, addresses[account.address].orEmpty(), ListPosition.getPosition(index, entries.size)) {
            onSelect(wallet, account)
        }
    }
}

@Composable
private fun WalletRecipient(
    wallet: Wallet,
    account: Account,
    address: String,
    listPosition: ListPosition,
    onClick: () -> Unit
) {
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(wallet.name) },
        data = {
            PropertyDataText(
                address,
                badge = { DataBadgeChevron() },
            )
        },
        listPosition = listPosition,
    )
}