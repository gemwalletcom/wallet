package com.gemwallet.android.application.contacts.cases

import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow

interface GetContacts {
    fun getContacts(): Flow<List<ContactData>>

    fun getContactRecipients(chain: Chain): Flow<List<ContactRecipient>>

    suspend fun getContact(id: String): ContactData?
}
