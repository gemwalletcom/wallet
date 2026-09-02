package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.SyncMissingAssets
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetsService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SyncMissingAssetsImpl(
    private val assetsService: GemAssetsService,
) : SyncMissingAssets {

    override suspend fun syncMissingAssets(assetIds: List<AssetId>): List<AssetId> = withContext(Dispatchers.IO) {
        assetsService.syncMissingAssets(assetIds.map { it.toIdentifier() }).mapNotNull { it.toAssetId() }
    }
}
