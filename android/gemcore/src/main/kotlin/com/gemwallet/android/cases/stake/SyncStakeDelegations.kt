package com.gemwallet.android.cases.stake

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId

interface SyncStakeDelegations {
    suspend fun sync(walletId: WalletId, chain: Chain, address: String, apr: Double)
}
