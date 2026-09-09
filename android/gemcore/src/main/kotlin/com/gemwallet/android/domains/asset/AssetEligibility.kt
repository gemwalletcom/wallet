package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import uniffi.gemstone.GemAssetFilter

fun List<GemAssetFilter>.toQueryFilters(): Set<AssetFilter> = mapNotNull { it.queryFilter() }.toSet()

fun List<GemAssetFilter>.eligible(items: List<AssetInfo>): List<AssetInfo> = items.filter { item -> all { item.matches(it) } }

private fun AssetInfo.matches(filter: GemAssetFilter): Boolean = when (filter) {
    GemAssetFilter.Enabled -> metadata.isEnabled
    GemAssetFilter.Buyable -> metadata.isBuyEnabled
    GemAssetFilter.Sellable -> metadata.isSellEnabled
    GemAssetFilter.Swappable -> metadata.isSwapEnabled
    GemAssetFilter.HasBalance -> balance.totalAmount != 0.0
    GemAssetFilter.HasAvailableBalance -> balance.balance.hasAvailable()
    is GemAssetFilter.ChainsOrAssetIds -> asset.id.chain in filter.chains.map { it.requireChain() } || asset.id.toIdentifier() in filter.assetIds
}

private fun GemAssetFilter.queryFilter(): AssetFilter? = when (this) {
    GemAssetFilter.Buyable -> AssetFilter.Buyable
    GemAssetFilter.Sellable -> AssetFilter.Sellable
    GemAssetFilter.Swappable -> AssetFilter.Swappable
    GemAssetFilter.HasBalance -> AssetFilter.HasBalance
    GemAssetFilter.HasAvailableBalance -> AssetFilter.HasAvailableBalance
    is GemAssetFilter.ChainsOrAssetIds -> AssetFilter.ChainsOrAssetIds(chains.map { it.requireChain() }, assetIds)
    GemAssetFilter.Enabled -> null
}
