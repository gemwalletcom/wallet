package com.gemwallet.android.cases.name

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

interface ResolveNameCase {
    suspend fun resolveName(name: String, chain: Chain): NameRecord?
}
