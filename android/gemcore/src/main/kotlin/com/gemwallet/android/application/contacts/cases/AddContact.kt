package com.gemwallet.android.application.contacts.cases

import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress

interface AddContact {
    suspend fun addContact(contact: Contact, addresses: List<ContactAddress>)
}
