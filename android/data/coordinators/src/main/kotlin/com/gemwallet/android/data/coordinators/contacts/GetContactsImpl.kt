package com.gemwallet.android.data.coordinators.contacts

import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.data.adapters.gemstone.GemstoneContactStore
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class GetContactsImpl(
    private val contactStore: GemstoneContactStore,
) : GetContacts {

    override fun getContacts(): Flow<List<ContactData>> = contactStore.observeContacts()

    override fun getContactRecipients(chain: Chain): Flow<List<ContactRecipient>> =
        contactStore.observeContactRecipients(chain)

    override suspend fun getContact(id: String): ContactData? = contactStore.getContact(id)
}
