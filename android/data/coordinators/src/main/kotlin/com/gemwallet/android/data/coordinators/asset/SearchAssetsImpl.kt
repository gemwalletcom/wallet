package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain

class SearchAssetsImpl(
    private val gemApiClient: GemApiClient,
) : SearchAssets {

    override suspend fun search(
        query: String,
        chains: List<Chain>,
        tags: List<AssetTag>,
    ): List<AssetBasic> {
        return gemApiClient.search(
            query = query,
            chains = chains.joinToString(",") { it.string },
            tags = tags.joinToString(",") { it.string },
        )
    }

    override suspend fun getAssets(assetIds: List<AssetId>): List<AssetBasic> {
        return gemApiClient.getAssets(assetIds)
    }
}
