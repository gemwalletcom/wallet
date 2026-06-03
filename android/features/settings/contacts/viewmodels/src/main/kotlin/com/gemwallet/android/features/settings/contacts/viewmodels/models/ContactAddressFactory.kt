package com.gemwallet.android.features.settings.contacts.viewmodels.models

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactAddress

fun newContactAddress(
    contactId: String,
    chain: Chain,
    address: String,
    memo: String?,
): ContactAddress = ContactAddress(
    id = "${contactId}_${chain.string}_${address}",
    contactId = contactId,
    address = address,
    chain = chain,
    memo = memo,
)
