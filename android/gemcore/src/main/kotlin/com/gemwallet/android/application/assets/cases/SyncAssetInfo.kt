package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

interface SyncAssetInfo {
    suspend fun syncAssetInfo(assetId: AssetId, wallet: Wallet)
}
