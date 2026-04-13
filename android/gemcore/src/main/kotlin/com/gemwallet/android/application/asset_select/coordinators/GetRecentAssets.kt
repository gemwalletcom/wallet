package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.flow.Flow

interface GetRecentAssets {
    operator fun invoke(
        types: List<RecentType>,
        filters: Set<AssetFilter> = emptySet(),
    ): Flow<List<Asset>>
}
