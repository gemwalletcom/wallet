package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress

fun mockContact(
    id: String = "contact-1",
    name: String = "John",
    description: String? = null,
    createdAt: Long = 0L,
    updatedAt: Long = 1L,
) = Contact(
    id = id,
    name = name,
    description = description,
    createdAt = createdAt,
    updatedAt = updatedAt,
)

fun mockContactAddress(
    id: String,
    chain: Chain,
    contactId: String = "contact-1",
    address: String = "address-$id",
    memo: String? = null,
) = ContactAddress(
    id = id,
    contactId = contactId,
    address = address,
    chain = chain,
    memo = memo,
)
