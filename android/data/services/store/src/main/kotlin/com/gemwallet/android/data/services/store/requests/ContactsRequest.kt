package com.gemwallet.android.data.services.store.requests

import com.gemwallet.android.data.services.store.database.ContactsDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class ContactsRequest @Inject constructor(private val contactsDao: ContactsDao) {

    operator fun invoke(): Flow<List<ContactData>> = contactsDao.getContacts().map { contacts -> contacts.map { it.toModel() } }
}
