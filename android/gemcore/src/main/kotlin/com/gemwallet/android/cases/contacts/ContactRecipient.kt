package com.gemwallet.android.cases.contacts

import com.wallet.core.primitives.Chain

data class ContactRecipient(
    val name: String,
    val address: String,
    val chain: Chain,
    val memo: String?,
)
