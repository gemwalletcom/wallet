package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemRecentActivity
import uniffi.gemstone.GemRecentActivityStore
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.RecentActivityType

class GemstoneRecentActivityStore(
    private val recentAssetsService: RecentAssetsService,
) : GemRecentActivityStore {

    override suspend fun add(activity: GemRecentActivity, walletId: String) {
        recentAssetsService.addRecentActivity(
            assetId = activity.assetId.toAssetId() ?: return,
            walletId = walletId,
            type = activity.activityType.toPrimitives(),
            toAssetId = activity.toAssetId?.toAssetId(),
        )
    }

    override suspend fun clear(walletId: String, types: List<RecentActivityType>) {
        recentAssetsService.clearRecentAssets(WalletId(walletId), types.map { it.toPrimitives() })
    }
}
