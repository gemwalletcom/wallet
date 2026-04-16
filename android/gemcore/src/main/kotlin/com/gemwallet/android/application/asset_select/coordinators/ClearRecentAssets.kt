package com.gemwallet.android.application.asset_select.coordinators

import com.gemwallet.android.model.RecentType

interface ClearRecentAssets {
    suspend operator fun invoke(types: List<RecentType> = RecentType.entries)
}
