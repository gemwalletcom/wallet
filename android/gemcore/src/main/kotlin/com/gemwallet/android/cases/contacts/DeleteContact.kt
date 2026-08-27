package com.gemwallet.android.cases.contacts

import com.wallet.core.primitives.Contact

interface DeleteContact {
    suspend fun deleteContact(contact: Contact)
}
