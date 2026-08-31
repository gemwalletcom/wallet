package com.gemwallet.android.application.contacts.cases

import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemContactAvatar

interface SaveContact {
    suspend fun saveContact(
        id: String,
        existing: Contact?,
        name: String,
        description: String,
        avatar: GemContactAvatar,
        addresses: List<ContactAddress>,
    ): Contact

    suspend fun saveAddresses(contact: Contact, addresses: List<ContactAddress>)
}
