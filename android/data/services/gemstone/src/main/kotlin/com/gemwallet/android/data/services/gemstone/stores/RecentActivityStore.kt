package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.RecentType
import uniffi.gemstone.GemRecentActivity
import uniffi.gemstone.GemRecentActivityStore
import uniffi.gemstone.RecentActivityType

class GemstoneRecentActivityStore(
    private val recentAssetsService: RecentAssetsService,
) : GemRecentActivityStore {

    override suspend fun add(activity: GemRecentActivity, walletId: String) {
        val assetId = activity.assetId.toAssetId() ?: return
        recentAssetsService.addRecentActivity(
            assetId = assetId,
            walletId = walletId,
            type = activity.activityType.toRecentType() ?: return,
            toAssetId = activity.toAssetId?.toAssetId(),
        )
    }
}

private fun RecentActivityType.toRecentType(): RecentType? = when (this) {
    RecentActivityType.SEARCH -> RecentType.Search
    RecentActivityType.TRANSFER -> RecentType.Send
    RecentActivityType.RECEIVE -> RecentType.Receive
    RecentActivityType.FIAT_BUY -> RecentType.Buy
    RecentActivityType.SWAP -> RecentType.Swap
    RecentActivityType.FIAT_SELL,
    RecentActivityType.SWAP_SELECT,
    RecentActivityType.PERPETUAL -> null
}
