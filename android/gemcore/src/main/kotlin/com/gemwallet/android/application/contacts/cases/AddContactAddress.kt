package com.gemwallet.android.application.contacts.cases

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactAddress

interface AddContactAddress {
    fun addAddress(
        addresses: List<ContactAddress>,
        contactId: String,
        chain: Chain,
        address: String,
        memo: String?,
        replacingId: String?,
    ): List<ContactAddress>

    fun defaultChain(): Chain
}
