package com.gemwallet.android.data.repositories.contacts

import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.gemwallet.android.cases.contacts.DeleteContact
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import com.wallet.core.primitives.ContactData
import com.wallet.core.primitives.VerificationStatus
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class ContactsRepository(
    private val contactsDao: ContactsDao,
    private val addressesDao: AddressesDao,
) : GetContacts, AddContact, UpdateContact, DeleteContact {

    override fun getContacts(): Flow<List<ContactData>> = contactsDao.getContacts()
        .map { contacts -> contacts.map { it.toModel() } }

    override fun getContactRecipients(chain: Chain): Flow<List<ContactRecipient>> =
        contactsDao.getContactRecipients(chain).map { rows -> rows.map { it.toModel() } }

    override suspend fun getContact(id: String): ContactData? = contactsDao.getContact(id)?.toModel()

    override suspend fun addContact(contact: Contact, addresses: List<ContactAddress>) {
        contactsDao.addContact(contact.toRecord(), addresses.map { it.toRecord() })
        syncAddressNames(contact, addresses)
    }

    override suspend fun updateContact(contact: Contact, addresses: List<ContactAddress>) {
        val newIds = addresses.map { it.id }.toSet()
        val removed = contactsDao.getAddresses(contact.id).filter { it.id !in newIds }

        contactsDao.updateContact(contact.toRecord(), removed.map { it.id }, addresses.map { it.toRecord() })
        removed.forEach { addressesDao.delete(it.chain, it.address, AddressType.Contact) }
        syncAddressNames(contact, addresses)
    }

    override suspend fun deleteContact(id: String) {
        contactsDao.getAddresses(id).forEach { address ->
            addressesDao.delete(address.chain, address.address, AddressType.Contact)
        }
        contactsDao.deleteContact(id)
    }

    private suspend fun syncAddressNames(contact: Contact, addresses: List<ContactAddress>) {
        if (addresses.isEmpty()) return
        val addressNames = addresses.map { address ->
            AddressName(
                chain = address.chain,
                address = address.address,
                name = contact.name,
                type = AddressType.Contact,
                status = VerificationStatus.Verified,
            )
        }
        runCatching { addressesDao.insert(addressNames.toRecord()) }
    }
}
