package com.gemwallet.android.application.stake.cases

import com.wallet.core.primitives.Chain

interface SyncStakeDelegations {
    suspend fun sync(chain: Chain)
}
