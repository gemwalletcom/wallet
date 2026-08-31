package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.AssetId

interface SyncAssetTransactions {
    suspend fun syncAssetTransactions(assetId: AssetId)
}
