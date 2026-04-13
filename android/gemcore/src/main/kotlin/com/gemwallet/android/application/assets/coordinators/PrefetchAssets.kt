package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId

interface PrefetchAssets {
    suspend fun prefetchAssets(assetIds: List<AssetId>)
}
