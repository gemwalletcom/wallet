package com.gemwallet.android.cases.stake

import com.wallet.core.primitives.Chain

interface SyncStakeDelegations {
    suspend fun sync(walletId: String, chain: Chain, address: String, apr: Double)
}
