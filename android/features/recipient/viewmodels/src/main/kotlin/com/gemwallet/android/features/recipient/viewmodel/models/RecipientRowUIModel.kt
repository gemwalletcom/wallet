package com.gemwallet.android.features.recipient.viewmodel.models

import android.content.Context
import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListSection
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemAddressFormatStyle
import uniffi.gemstone.GemAddressServiceInterface
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientSection

data class RecipientRowUIModel(
    val recipient: GemRecipient,
    val memo: String?,
    val model: ListItemModel,
)

internal fun GemRecipientSection.uiSection(
    id: String,
    context: Context,
    addressService: GemAddressServiceInterface,
    contacts: List<ContactRecipient>,
    chain: Chain,
): ListSection<RecipientRowUIModel> = ListSection(
    id = id,
    title = context.getString(stringRes()),
    items = when (this) {
        is GemRecipientSection.Contacts -> contactRows(contacts, addressService)
        is GemRecipientSection.Pinned -> walletRows(wallets.map { it.toPrimitives() }, chain, addressService)
        is GemRecipientSection.Wallets -> walletRows(wallets.map { it.toPrimitives() }, chain, addressService)
        is GemRecipientSection.ViewWallets -> walletRows(wallets.map { it.toPrimitives() }, chain, addressService)
    },
)

private fun contactRows(contacts: List<ContactRecipient>, addressService: GemAddressServiceInterface): List<RecipientRowUIModel> {
    val addresses = addressService.formatAll(contacts.map { ChainAddress(it.chain, it.address).toGem() }, GemAddressFormatStyle.Short)
    return contacts.zip(addresses) { contact, address ->
        RecipientRowUIModel(
            recipient = GemRecipient(address = contact.address, name = contact.name),
            memo = contact.memo ?: "",
            model = ListItemModel(title = contact.name, subtitle = address),
        )
    }
}

private fun walletRows(wallets: List<com.wallet.core.primitives.Wallet>, chain: Chain, addressService: GemAddressServiceInterface): List<RecipientRowUIModel> {
    val entries = wallets.mapNotNull { wallet -> wallet.getAccount(chain)?.let { wallet to it } }
    val addresses = addressService.formatAll(entries.map { (_, account) -> ChainAddress(account.chain, account.address).toGem() }, GemAddressFormatStyle.Short)
    return entries.zip(addresses) { (wallet, account), address ->
        RecipientRowUIModel(
            recipient = GemRecipient(address = account.address, name = wallet.name),
            memo = null,
            model = ListItemModel(title = wallet.name, subtitle = address),
        )
    }
}
