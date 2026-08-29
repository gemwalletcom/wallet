package com.gemwallet.android.data.coordinators.contacts

import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.DeleteContact
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemContactService

class AddContactImpl(
    private val contactService: GemContactService,
) : AddContact {

    override suspend fun addContact(contact: Contact, addresses: List<ContactAddress>) =
        contactService.addContact(contact.toJson(), addresses.map { it.toJson() })
}

class UpdateContactImpl(
    private val contactService: GemContactService,
) : UpdateContact {

    override suspend fun updateContact(contact: Contact, addresses: List<ContactAddress>) =
        contactService.updateContact(contact.toJson(), addresses.map { it.toJson() })
}

class DeleteContactImpl(
    private val contactService: GemContactService,
) : DeleteContact {

    override suspend fun deleteContact(contact: Contact) = contactService.deleteContact(contact.toJson())
}
