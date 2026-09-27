package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.DbRecentActivity
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemRecentActivity
import uniffi.gemstone.GemRecentActivityStore
import uniffi.gemstone.RecentActivityType

class GemstoneRecentActivityStore(private val assetsDao: AssetsDao) : GemRecentActivityStore {

    override suspend fun add(activity: GemRecentActivity, walletId: String) {
        assetsDao.addRecentActivity(
            DbRecentActivity(
                assetId = (activity.assetId.toAssetId() ?: return).toIdentifier(),
                walletId = walletId,
                toAssetId = activity.toAssetId?.toAssetId()?.toIdentifier(),
                type = activity.activityType.toPrimitives(),
                addedAt = System.currentTimeMillis(),
            ),
        )
    }

    override suspend fun clear(walletId: String, types: List<RecentActivityType>) {
        assetsDao.clearRecentAssets(walletId, types.map { it.toPrimitives() })
    }
}
