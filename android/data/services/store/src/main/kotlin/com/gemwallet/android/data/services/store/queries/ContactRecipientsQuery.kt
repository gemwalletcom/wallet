package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.data.services.store.database.ContactsDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class ContactRecipientsQuery @Inject constructor(private val contactsDao: ContactsDao) {

    operator fun invoke(chain: Chain): Flow<List<ContactRecipient>> = contactsDao.getContactRecipients(chain).map { rows -> rows.map { it.toModel() } }
}
