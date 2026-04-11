package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId

class PrefetchAssetsImpl(
    private val gemApiClient: GemApiClient,
    private val assetsRepository: AssetsRepository,
) : PrefetchAssets {

    override suspend fun prefetchAssets(assetIds: List<AssetId>) {
        val requestedAssetIds = assetIds.distinct()
        val existingAssetIds = assetsRepository.hasAssets(requestedAssetIds)
        val missingAssetIds = requestedAssetIds.filterNot(existingAssetIds::contains)

        if (missingAssetIds.isEmpty()) {
            return
        }

        assetsRepository.add(loadAssets(missingAssetIds))
    }

    private suspend fun loadAssets(assetIds: List<AssetId>): List<AssetBasic> {
        if (assetIds.isEmpty()) return emptyList()

        return runCatching { gemApiClient.getAssets(assetIds) }
            .getOrDefault(emptyList())
    }
}
