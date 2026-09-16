package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.model.AssetFilter
import uniffi.gemstone.GemAssetFilter

fun List<GemAssetFilter>.toQueryFilters(): Set<AssetFilter> = mapNotNull { it.queryFilter() }.toSet()

private fun GemAssetFilter.queryFilter(): AssetFilter? = when (this) {
    GemAssetFilter.Buyable -> AssetFilter.Buyable
    GemAssetFilter.Sellable -> AssetFilter.Sellable
    GemAssetFilter.Swappable -> AssetFilter.Swappable
    GemAssetFilter.HasBalance -> AssetFilter.HasBalance
    GemAssetFilter.HasAvailableBalance -> AssetFilter.HasAvailableBalance
    is GemAssetFilter.ChainsOrAssetIds -> AssetFilter.ChainsOrAssetIds(chains.map { it.requireChain() }, assetIds)
    GemAssetFilter.Enabled -> null
}
