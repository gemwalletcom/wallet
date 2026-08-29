package com.gemwallet.android.application.contacts.cases

import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress

interface UpdateContact {
    suspend fun updateContact(contact: Contact, addresses: List<ContactAddress>)
}
