package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import kotlinx.coroutines.flow.Flow

class GetRecentAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : GetRecentAssets {
    override fun invoke(request: RecentAssetsRequest): Flow<List<RecentAsset>> =
        assetsRepository.getRecentAssets(request)
}
