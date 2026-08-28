package com.gemwallet.android.application.asset_select.cases

import com.gemwallet.android.model.RecentType

interface ClearRecentAssets {
    suspend operator fun invoke(types: List<RecentType> = RecentType.entries)
}
