package com.gemwallet.android.application.transactions.coordinators

import com.wallet.core.primitives.AssetId

interface SyncAssetTransactions {
    suspend fun syncAssetTransactions(assetId: AssetId)
}
