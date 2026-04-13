package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.RecentType
import kotlinx.coroutines.flow.Flow

class GetRecentAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : GetRecentAssets {
    override fun invoke(types: List<RecentType>): Flow<List<AssetInfo>> =
        assetsRepository.getRecentActivities(types)
}
