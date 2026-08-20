package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

interface SearchAssets {
    suspend fun searchAssets(
        query: String,
        chains: List<Chain> = emptyList(),
    ): List<AssetBasic>

    suspend fun getAssets(assetIds: List<AssetId>): List<AssetBasic>
}
