package com.gemwallet.android.cases.contacts

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow

interface GetContacts {
    fun getContacts(): Flow<List<ContactData>>

    fun getContactRecipients(chain: Chain): Flow<List<ContactRecipient>>

    suspend fun getContact(id: String): ContactData?
}
