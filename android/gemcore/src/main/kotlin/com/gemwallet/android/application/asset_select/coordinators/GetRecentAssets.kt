package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.RecentType
import kotlinx.coroutines.flow.Flow

interface GetRecentAssets {
    fun getRecentActivities(types: List<RecentType>): Flow<List<AssetInfo>>
}
