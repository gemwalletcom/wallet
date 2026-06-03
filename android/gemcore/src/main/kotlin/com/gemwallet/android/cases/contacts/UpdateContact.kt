package com.gemwallet.android.cases.contacts

import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress

interface UpdateContact {
    suspend fun updateContact(contact: Contact, addresses: List<ContactAddress>)
}
