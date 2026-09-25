package com.gemwallet.android.data.services.store.requests

import com.gemwallet.android.data.services.store.database.ContactsDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.ContactData
import javax.inject.Inject

class ContactRequest @Inject constructor(private val contactsDao: ContactsDao) {

    suspend operator fun invoke(id: String): ContactData? = contactsDao.getContact(id)?.toModel()
}
