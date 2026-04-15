package com.gemwallet.android.model

data class RecentAssetsRequest(
    val types: List<RecentType> = RecentType.entries,
    val filters: Set<AssetFilter> = emptySet(),
)
