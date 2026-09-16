package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.gemwallet.android.application.contacts.values.ContactRecipient
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
import com.wallet.core.primitives.ChainAddress

@Composable
fun rememberContactAddresses(contacts: List<ContactRecipient>): Map<String, String> = rememberFormattedAddresses(
    remember(contacts) { contacts.map { ChainAddress(chain = it.chain, address = it.address) } }
)

fun LazyListScope.contactsSection(
    section: GemRecipientSection,
    contacts: List<ContactRecipient>,
    addresses: Map<String, String>,
    onSelect: (ContactRecipient) -> Unit,
) {
    if (contacts.isEmpty()) {
        return
    }
    item {
        SubheaderItem(section.stringRes())
    }
    itemsIndexed(contacts) { index, contact ->
        ContactRecipientItem(contact, addresses[contact.address].orEmpty(), ListPosition.getPosition(index, contacts.size)) {
            onSelect(contact)
        }
    }
}

@Composable
private fun ContactRecipientItem(
    contact: ContactRecipient,
    address: String,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(contact.name) },
        data = {
            PropertyDataText(
                address,
                badge = { DataBadgeChevron() },
            )
        },
        listPosition = listPosition,
    )
}
