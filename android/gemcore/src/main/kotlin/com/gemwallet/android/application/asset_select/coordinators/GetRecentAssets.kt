package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import kotlinx.coroutines.flow.Flow

interface GetRecentAssets {
    operator fun invoke(request: RecentAssetsRequest = RecentAssetsRequest()): Flow<List<RecentAsset>>
}
