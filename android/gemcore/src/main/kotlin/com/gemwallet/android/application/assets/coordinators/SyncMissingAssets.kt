package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId

interface SyncMissingAssets {
    suspend fun syncMissingAssets(assetIds: List<AssetId>): List<AssetId>
}
