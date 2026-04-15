package com.gemwallet.android.cases.assets

import com.gemwallet.android.model.RecentAssetsRequest
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.flow.Flow

interface GetRecent {
    fun getRecentAssets(request: RecentAssetsRequest = RecentAssetsRequest()): Flow<List<Asset>>
}
