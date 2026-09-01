package com.gemwallet.android.application.asset_select.cases

import com.wallet.core.primitives.RecentActivityType

interface ClearRecentAssets {
    suspend operator fun invoke(types: List<RecentActivityType> = RecentActivityType.entries)
}
