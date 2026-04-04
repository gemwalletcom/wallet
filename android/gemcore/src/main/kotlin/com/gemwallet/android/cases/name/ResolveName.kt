package com.gemwallet.android.cases.name

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

interface ResolveName {
    suspend fun resolveName(name: String, chain: Chain): NameRecord?

    fun canResolveName(name: String): Boolean {
        val nameParts = name.split(".")
        return nameParts.size >= 2 && !nameParts.lastOrNull().isNullOrEmpty()
    }
}
