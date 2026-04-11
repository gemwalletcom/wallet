package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.RecentType
import kotlinx.coroutines.flow.Flow

interface GetRecentAssets {
    operator fun invoke(types: List<RecentType>): Flow<List<AssetInfo>>
}
