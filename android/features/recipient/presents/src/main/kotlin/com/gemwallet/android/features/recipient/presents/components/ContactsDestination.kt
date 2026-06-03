package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition

fun LazyListScope.contactsDestination(
    contacts: List<ContactRecipient>,
    onSelect: (ContactRecipient) -> Unit,
) {
    if (contacts.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.contacts_title)
    }
    itemsIndexed(contacts) { index, contact ->
        ContactRecipientItem(contact, ListPosition.getPosition(index, contacts.size)) {
            onSelect(contact)
        }
    }
}

@Composable
private fun ContactRecipientItem(
    contact: ContactRecipient,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(contact.name) },
        data = {
            PropertyDataText(
                AddressFormatter(contact.address, chain = contact.chain).value(),
                badge = { DataBadgeChevron() },
            )
        },
        listPosition = listPosition,
    )
}
