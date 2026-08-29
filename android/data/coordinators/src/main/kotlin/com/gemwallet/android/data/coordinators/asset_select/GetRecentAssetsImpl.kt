package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.data.adapters.assets.RecentAssetsService
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import kotlinx.coroutines.flow.Flow

class GetRecentAssetsImpl(
    private val recentAssetsService: RecentAssetsService,
) : GetRecentAssets {
    override fun invoke(request: RecentAssetsRequest): Flow<List<RecentAsset>> =
        recentAssetsService.getRecentAssets(request)
}
