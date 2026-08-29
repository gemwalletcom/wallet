package com.gemwallet.android.application.recipient.cases

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

interface GetNameRecord {
    suspend fun getNameRecord(name: String, chain: Chain): NameRecord?

    fun isNameSupported(name: String): Boolean
}
