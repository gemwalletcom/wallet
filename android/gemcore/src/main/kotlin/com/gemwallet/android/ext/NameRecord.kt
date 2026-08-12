package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

fun NameRecord.matchesRecipient(name: String, address: String, chain: Chain): Boolean =
    this.name == name && this.address == address && this.chain == chain
