package com.gemwallet.android.data.repositories.contacts

import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.gemwallet.android.cases.contacts.DeleteContact
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemContactService

class ContactsRepository(
    private val contactsDao: ContactsDao,
    private val contactService: GemContactService,
) : GetContacts, AddContact, UpdateContact, DeleteContact {

    override fun getContacts(): Flow<List<ContactData>> = contactsDao.getContacts()
        .map { contacts -> contacts.map { it.toModel() } }

    override fun getContactRecipients(chain: Chain): Flow<List<ContactRecipient>> =
        contactsDao.getContactRecipients(chain).map { rows -> rows.map { it.toModel() } }

    override suspend fun getContact(id: String): ContactData? = contactsDao.getContact(id)?.toModel()

    override suspend fun addContact(contact: Contact, addresses: List<ContactAddress>) =
        contactService.addContact(contact.toJson(), addresses.map { it.toJson() })

    override suspend fun updateContact(contact: Contact, addresses: List<ContactAddress>) =
        contactService.updateContact(contact.toJson(), addresses.map { it.toJson() })

    override suspend fun deleteContact(contact: Contact) = contactService.deleteContact(contact.toJson())
}
