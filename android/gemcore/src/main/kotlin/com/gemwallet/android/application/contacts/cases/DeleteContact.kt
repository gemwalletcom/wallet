package com.gemwallet.android.application.contacts.cases

import com.wallet.core.primitives.Contact

interface DeleteContact {
    suspend fun deleteContact(contact: Contact)
}
