package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.assetActionFilters

fun GemAssetAction.filters(): List<GemAssetFilter> = assetActionFilters(this)

fun GemAssetAction.recentFilters(): Set<AssetFilter> = filters().mapNotNull { it.recentFilter() }.toSet()

fun GemAssetAction.eligible(items: List<AssetInfo>): List<AssetInfo> {
    val filters = filters()
    return items.filter { item -> filters.all { item.matches(it) } }
}

private fun AssetInfo.matches(filter: GemAssetFilter): Boolean = when (filter) {
    GemAssetFilter.ENABLED -> metadata?.isEnabled != false
    GemAssetFilter.BUYABLE -> metadata?.isBuyEnabled == true
    GemAssetFilter.SELLABLE -> metadata?.isSellEnabled == true
    GemAssetFilter.SWAPPABLE -> metadata?.isSwapEnabled == true
    GemAssetFilter.HAS_BALANCE -> balance.totalAmount != 0.0
    GemAssetFilter.HAS_AVAILABLE_BALANCE -> balance.balance.hasAvailable()
}

private fun GemAssetFilter.recentFilter(): AssetFilter? = when (this) {
    GemAssetFilter.BUYABLE -> AssetFilter.Buyable
    GemAssetFilter.SWAPPABLE -> AssetFilter.Swappable
    GemAssetFilter.HAS_BALANCE, GemAssetFilter.HAS_AVAILABLE_BALANCE -> AssetFilter.HasBalance
    GemAssetFilter.ENABLED, GemAssetFilter.SELLABLE -> null
}
