package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.ContactsDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemContactStore

class GemstoneContactStore(private val contactsDao: ContactsDao) : GemContactStore {

    override suspend fun getAddresses(contactId: String): List<uniffi.gemstone.ContactAddress> = contactsDao.getAddresses(contactId).map { it.toModel().toGem() }

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
}
