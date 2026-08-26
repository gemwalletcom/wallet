package com.gemwallet.android.data.repositories.contacts

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemContactStore

class GemstoneContactStore(
    private val contactsDao: ContactsDao,
    private val addressesDao: AddressesDao,
) : GemContactStore {

    override suspend fun getAddressIds(contactId: String): List<String> =
        contactsDao.getAddresses(contactId).map { it.id }

    override suspend fun saveContact(contact: String, addresses: List<String>) {
        contactsDao.addContact(
            contact.decodeJson<Contact>().toRecord(),
            addresses.map { it.decodeJson<ContactAddress>().toRecord() },
        )
    }

    override suspend fun updateContact(contact: String, addresses: List<String>, deleteAddressIds: List<String>) {
        val record = contact.decodeJson<Contact>()
        val removed = contactsDao.getAddresses(record.id).filter { it.id in deleteAddressIds }
        contactsDao.updateContact(record.toRecord(), deleteAddressIds, addresses.map { it.decodeJson<ContactAddress>().toRecord() })
        removed.forEach { addressesDao.delete(it.chain, it.address, AddressType.Contact) }
    }

    override suspend fun deleteContact(contactId: String) {
        contactsDao.getAddresses(contactId).forEach { address ->
            addressesDao.delete(address.chain, address.address, AddressType.Contact)
        }
        contactsDao.deleteContact(contactId)
    }
}
