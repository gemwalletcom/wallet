package com.gemwallet.android.cases.stake

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

interface SyncStakeDelegations {
    suspend fun sync(walletId: WalletId, assetId: AssetId, address: String, apr: Double)
}
