package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemAssetFilter

fun GemAssetAction.recentFilters(): Set<AssetFilter> = filters().mapNotNull { it.recentFilter() }.toSet()

fun GemAssetAction.queryFilters(): Set<AssetFilter> = filters().mapNotNull { it.queryFilter() }.toSet()

fun GemAssetAction.eligible(items: List<AssetInfo>): List<AssetInfo> {
    val filters = filters()
    return items.filter { item -> filters.all { item.matches(it) } }
}

private fun AssetInfo.matches(filter: GemAssetFilter): Boolean = when (filter) {
    GemAssetFilter.ENABLED -> metadata.isEnabled
    GemAssetFilter.BUYABLE -> metadata.isBuyEnabled
    GemAssetFilter.SELLABLE -> metadata.isSellEnabled
    GemAssetFilter.SWAPPABLE -> metadata.isSwapEnabled
    GemAssetFilter.HAS_BALANCE -> balance.totalAmount != 0.0
    GemAssetFilter.HAS_AVAILABLE_BALANCE -> balance.balance.hasAvailable()
}

private fun GemAssetFilter.queryFilter(): AssetFilter? = when (this) {
    GemAssetFilter.BUYABLE -> AssetFilter.Buyable
    GemAssetFilter.SELLABLE -> AssetFilter.Sellable
    GemAssetFilter.SWAPPABLE -> AssetFilter.Swappable
    GemAssetFilter.HAS_BALANCE -> AssetFilter.HasBalance
    GemAssetFilter.HAS_AVAILABLE_BALANCE -> AssetFilter.HasAvailableBalance
    GemAssetFilter.ENABLED -> null
}

private fun GemAssetFilter.recentFilter(): AssetFilter? = when (this) {
    GemAssetFilter.BUYABLE -> AssetFilter.Buyable
    GemAssetFilter.SWAPPABLE -> AssetFilter.Swappable
    GemAssetFilter.HAS_BALANCE -> AssetFilter.HasBalance
    GemAssetFilter.HAS_AVAILABLE_BALANCE -> AssetFilter.HasAvailableBalance
    GemAssetFilter.ENABLED, GemAssetFilter.SELLABLE -> null
}
