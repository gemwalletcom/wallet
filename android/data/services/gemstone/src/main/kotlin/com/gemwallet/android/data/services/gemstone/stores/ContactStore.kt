package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemContactStore

class GemstoneContactStore(
    private val contactsDao: ContactsDao,
) : GemContactStore {

    override suspend fun getAddresses(contactId: String): List<uniffi.gemstone.ContactAddress> =
        contactsDao.getAddresses(contactId).map { it.toModel().toGem() }

    override suspend fun saveContact(contact: uniffi.gemstone.Contact, addresses: List<uniffi.gemstone.ContactAddress>) {
        contactsDao.addContact(
            contact.toPrimitives().toRecord(),
            addresses.map { it.toPrimitives().toRecord() },
        )
    }

    override suspend fun updateContact(contact: uniffi.gemstone.Contact, addresses: List<uniffi.gemstone.ContactAddress>, deleteAddressIds: List<String>) {
        contactsDao.updateContact(contact.toPrimitives().toRecord(), deleteAddressIds, addresses.map { it.toPrimitives().toRecord() })
    }

    override suspend fun deleteContact(contactId: String) = contactsDao.deleteContact(contactId)

    fun observeContacts(): Flow<List<ContactData>> = contactsDao.getContacts().map { contacts -> contacts.map { it.toModel() } }

    fun observeContactRecipients(chain: Chain): Flow<List<ContactRecipient>> =
        contactsDao.getContactRecipients(chain).map { rows -> rows.map { it.toModel() } }

    suspend fun getContact(id: String): ContactData? = contactsDao.getContact(id)?.toModel()
}
