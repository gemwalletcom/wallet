package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.coordinators.ClearRecentAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.RecentType

class ClearRecentAssetsImpl(
    private val assetsRepository: AssetsRepository,
) : ClearRecentAssets {
    override suspend fun invoke(types: List<RecentType>) =
        assetsRepository.clearRecentAssets(types)
}
