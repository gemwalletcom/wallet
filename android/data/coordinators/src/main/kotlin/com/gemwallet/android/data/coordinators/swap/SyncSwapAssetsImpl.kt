package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.swap.coordinators.GetSwapAssets
import com.gemwallet.android.application.swap.coordinators.SyncSwapAssets
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.ConfigVersions
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive

class SyncSwapAssetsImpl(
    private val configStore: ConfigStore,
    private val getSwapAssets: GetSwapAssets,
    private val assetsRepository: AssetsRepository,
    private val availabilityService: AssetsAvailabilityService,
    private val prefetchAssets: PrefetchAssets,
) : SyncSwapAssets {

    override suspend fun invoke(versions: ConfigVersions) {
        try {
            if (!shouldSync(versions.swapAssets)) {
                return
            }
            val swapAssets = getSwapAssets()
            val synced = swapAssets.assetIds.distinct().chunked(ASSET_BATCH_SIZE).map { batch ->
                val assetIds = batch.mapNotNull(String::toAssetId)
                prefetchAssets.prefetchAssets(assetIds)
                availabilityService.updateSwapAvailable(batch)
                assetsRepository.hasAssets(assetIds).size == assetIds.size
            }
            if (synced.all { it }) {
                configStore.putInt(SWAP_ASSETS_VERSION, swapAssets.version.toInt())
            }
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    private fun shouldSync(remoteVersion: Int): Boolean {
        val currentVersion = configStore.getInt(SWAP_ASSETS_VERSION)
        return currentVersion <= 0 || currentVersion < remoteVersion
    }

    private companion object {
        const val SWAP_ASSETS_VERSION = "swap-assets-version"
        const val ASSET_BATCH_SIZE = 500
    }
}
