package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetsService

class PrefetchAssetsImpl(
    private val assetsService: GemAssetsService,
    private val assetsRepository: AssetsRepository,
) : PrefetchAssets {

    override suspend fun prefetchAssets(assetIds: List<AssetId>): List<AssetId> {
        val requestedAssetIds = assetIds.distinct()
        val existingAssetIds = assetsRepository.hasAssets(requestedAssetIds)
        val missingAssetIds = requestedAssetIds.filterNot(existingAssetIds::contains)

        if (missingAssetIds.isEmpty()) {
            return emptyList()
        }

        val loadedAssets = loadAssets(missingAssetIds)
        assetsRepository.add(loadedAssets)
        return loadedAssets.map { it.asset.id }
    }

    private suspend fun loadAssets(assetIds: List<AssetId>): List<AssetBasic> {
        if (assetIds.isEmpty()) return emptyList()

        return runCatching {
            assetsService.getAssets(assetIds.map { it.toIdentifier() }, null).map { it.decodeJson<AssetBasic>() }
        }.getOrDefault(emptyList())
    }
}
