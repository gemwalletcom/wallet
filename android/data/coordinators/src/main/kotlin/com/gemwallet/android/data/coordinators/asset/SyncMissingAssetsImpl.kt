package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SyncMissingAssets
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetsService

class SyncMissingAssetsImpl(
    private val assetsService: GemAssetsService,
) : SyncMissingAssets {

    override suspend fun syncMissingAssets(assetIds: List<AssetId>): List<AssetId> =
        assetsService.syncMissingAssets(assetIds.map { it.toIdentifier() }).mapNotNull { it.toAssetId() }
}
