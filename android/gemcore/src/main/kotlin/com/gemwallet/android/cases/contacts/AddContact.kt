package com.gemwallet.android.cases.contacts

import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress

interface AddContact {
    suspend fun addContact(contact: Contact, addresses: List<ContactAddress>)
}
