package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.flow.Flow

class GetRecentAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : GetRecentAssets {
    override fun invoke(types: List<RecentType>, filters: Set<AssetFilter>): Flow<List<Asset>> =
        assetsRepository.getRecentActivities(types, filters)
}
