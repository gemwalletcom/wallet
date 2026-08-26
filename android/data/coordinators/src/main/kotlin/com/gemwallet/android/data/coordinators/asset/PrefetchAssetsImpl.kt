package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetsService

class PrefetchAssetsImpl(
    private val assetsService: GemAssetsService,
) : PrefetchAssets {

    override suspend fun prefetchAssets(assetIds: List<AssetId>): List<AssetId> =
        assetsService.prefetchAssets(assetIds.map { it.toIdentifier() }).mapNotNull { it.toAssetId() }
}
