package com.gemwallet.android.application.stake.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

interface SyncStakeDelegations {
    suspend fun sync(walletId: WalletId, assetId: AssetId, address: String)
}
