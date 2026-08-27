package com.gemwallet.android.cases.name

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

interface GetNameRecord {
    suspend fun getNameRecord(name: String, chain: Chain): NameRecord?

    fun isNameSupported(name: String): Boolean
}
