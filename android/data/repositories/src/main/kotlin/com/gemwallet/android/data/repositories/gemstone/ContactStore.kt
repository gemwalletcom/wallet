package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemContactStore

class GemstoneContactStore(
    private val contactsDao: ContactsDao,
) : GemContactStore {

    override suspend fun getAddresses(contactId: String): List<String> =
        contactsDao.getAddresses(contactId).map { it.toModel().toJson() }

    override suspend fun saveContact(contact: String, addresses: List<String>) {
        contactsDao.addContact(
            contact.decodeJson<Contact>().toRecord(),
            addresses.map { it.decodeJson<ContactAddress>().toRecord() },
        )
    }

    override suspend fun updateContact(contact: String, addresses: List<String>, deleteAddressIds: List<String>) {
        contactsDao.updateContact(contact.decodeJson<Contact>().toRecord(), deleteAddressIds, addresses.map { it.decodeJson<ContactAddress>().toRecord() })
    }

    override suspend fun deleteContact(contactId: String) = contactsDao.deleteContact(contactId)

    fun observeContacts(): Flow<List<ContactData>> = contactsDao.getContacts().map { contacts -> contacts.map { it.toModel() } }

    fun observeContactRecipients(chain: Chain): Flow<List<ContactRecipient>> =
        contactsDao.getContactRecipients(chain).map { rows -> rows.map { it.toModel() } }

    suspend fun getContact(id: String): ContactData? = contactsDao.getContact(id)?.toModel()
}
