package com.gemwallet.android.data.coordinators.contacts

import com.gemwallet.android.cases.contacts.ContactRecipient
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class GetContactsImpl(
    private val contactsDao: ContactsDao,
) : GetContacts {

    override fun getContacts(): Flow<List<ContactData>> = contactsDao.getContacts()
        .map { contacts -> contacts.map { it.toModel() } }

    override fun getContactRecipients(chain: Chain): Flow<List<ContactRecipient>> =
        contactsDao.getContactRecipients(chain).map { rows -> rows.map { it.toModel() } }

    override suspend fun getContact(id: String): ContactData? = contactsDao.getContact(id)?.toModel()
}
