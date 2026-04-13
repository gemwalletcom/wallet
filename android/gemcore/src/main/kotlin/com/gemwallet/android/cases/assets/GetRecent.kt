package com.gemwallet.android.cases.assets

import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.flow.Flow

interface GetRecent {
    fun getRecentActivities(
        type: List<RecentType>,
        filters: Set<AssetFilter> = emptySet(),
    ): Flow<List<Asset>>
}
