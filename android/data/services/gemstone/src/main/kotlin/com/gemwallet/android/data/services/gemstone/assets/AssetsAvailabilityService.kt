package com.gemwallet.android.data.services.gemstone.assets

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.domains.asset.calculateAvailabilityChanges
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AssetsAvailabilityService @Inject constructor(
    private val assetsDao: AssetsDao,
) {

    suspend fun updateBuyAvailable(assets: List<String>) {
        syncAvailability(
            currentEnabledAssetIds = assetsDao.getBuyAvailableAssetIds(),
            targetEnabledAssetIds = assets,
            setAvailability = assetsDao::setBuyAvailable,
        )
    }

    suspend fun updateSellAvailable(assets: List<String>) {
        syncAvailability(
            currentEnabledAssetIds = assetsDao.getSellAvailableAssetIds(),
            targetEnabledAssetIds = assets,
            setAvailability = assetsDao::setSellAvailable,
        )
    }

    suspend fun updateSwapAvailable(assets: List<String>) {
        syncAvailability(
            currentEnabledAssetIds = assetsDao.getSwapAvailableAssetIds(assets),
            targetEnabledAssetIds = assets,
            trackedAssetIds = assets,
            setAvailability = assetsDao::setSwapAvailable,
        )
    }

    private suspend fun syncAvailability(
        currentEnabledAssetIds: List<String>,
        targetEnabledAssetIds: List<String>,
        setAvailability: suspend (List<String>, Boolean) -> Unit,
        trackedAssetIds: List<String> = (currentEnabledAssetIds + targetEnabledAssetIds).distinct(),
    ) {
        val changes = calculateAvailabilityChanges(
            currentEnabledAssetIds = currentEnabledAssetIds,
            targetEnabledAssetIds = targetEnabledAssetIds,
            trackedAssetIds = trackedAssetIds,
        )

        if (changes.idsToDisable.isNotEmpty()) {
            setAvailability(changes.idsToDisable, false)
        }
        if (changes.idsToEnable.isNotEmpty()) {
            setAvailability(changes.idsToEnable, true)
        }
    }
}
