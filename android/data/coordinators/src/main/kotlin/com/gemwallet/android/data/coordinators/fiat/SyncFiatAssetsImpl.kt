package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.toAssetId
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive

class SyncFiatAssetsImpl(
    private val configStore: ConfigStore,
    private val getRemoteConfig: GetRemoteConfig,
    private val getBuyableFiatAssets: GetBuyableFiatAssets,
    private val getSellableFiatAssets: GetSellableFiatAssets,
    private val assetsRepository: AssetsRepository,
    private val prefetchAssets: PrefetchAssets,
) : SyncFiatAssets {

    override suspend fun invoke() {
        try {
            val versions = getRemoteConfig.getRemoteConfig().versions
            val buyAssets = if (shouldSync(FIAT_ON_RAMP_ASSETS_VERSION, versions.fiatOnRampAssets)) {
                getBuyableFiatAssets()
            } else {
                null
            }
            val sellAssets = if (shouldSync(FIAT_OFF_RAMP_ASSETS_VERSION, versions.fiatOffRampAssets)) {
                getSellableFiatAssets()
            } else {
                null
            }
            if (buyAssets == null && sellAssets == null) {
                return
            }

            val assetIds = listOfNotNull(buyAssets, sellAssets)
                .flatMap { it.assetIds }
                .mapNotNull(String::toAssetId)
                .distinct()

            if (assetIds.isNotEmpty()) {
                prefetchAssets.prefetchAssets(assetIds)
            }

            buyAssets?.let {
                assetsRepository.updateBuyAvailable(it.assetIds)
                configStore.putInt(FIAT_ON_RAMP_ASSETS_VERSION, it.version.toInt())
            }
            sellAssets?.let {
                assetsRepository.updateSellAvailable(it.assetIds)
                configStore.putInt(FIAT_OFF_RAMP_ASSETS_VERSION, it.version.toInt())
            }
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    private fun shouldSync(configKey: String, remoteVersion: Int): Boolean {
        val currentVersion = configStore.getInt(configKey)
        return currentVersion <= 0 || currentVersion < remoteVersion
    }

    private companion object {
        const val FIAT_ON_RAMP_ASSETS_VERSION = "fiat-on-ramp-assets-version"
        const val FIAT_OFF_RAMP_ASSETS_VERSION = "fiat-off-ramp-assets-version"
    }
}
