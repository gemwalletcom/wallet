package com.gemwallet.android.model

import com.wallet.core.primitives.RecentActivityType

data class RecentAssetsRequest(
    val types: List<RecentActivityType> = RecentActivityType.entries,
    val filters: Set<AssetFilter> = emptySet(),
    val limit: Int = 10,
)
