package com.gemwallet.android.application.assets.values

import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetFilter

sealed interface AssetsQueryFilter {
    data object Enabled : AssetsQueryFilter
    data object Buyable : AssetsQueryFilter
    data object Sellable : AssetsQueryFilter
    data object Swappable : AssetsQueryFilter
    data object HasBalance : AssetsQueryFilter
    data object HasAvailableBalance : AssetsQueryFilter
    data class Chains(val chains: List<Chain>) : AssetsQueryFilter
    data class ChainsOrAssets(val chains: List<Chain>, val assetIds: List<String>) : AssetsQueryFilter
}

fun Collection<AssetsQueryFilter>.chains(): List<Chain> = filterIsInstance<AssetsQueryFilter.Chains>().flatMap { it.chains }

fun Collection<AssetsQueryFilter>.chainsOrAssets(): AssetsQueryFilter.ChainsOrAssets? = filterIsInstance<AssetsQueryFilter.ChainsOrAssets>()
    .takeIf { it.isNotEmpty() }
    ?.let { filters -> AssetsQueryFilter.ChainsOrAssets(filters.flatMap { it.chains }, filters.flatMap { it.assetIds }) }

fun GemAssetFilter.toQueryFilter(): AssetsQueryFilter = when (this) {
    GemAssetFilter.Enabled -> AssetsQueryFilter.Enabled
    GemAssetFilter.Buyable -> AssetsQueryFilter.Buyable
    GemAssetFilter.Sellable -> AssetsQueryFilter.Sellable
    GemAssetFilter.Swappable -> AssetsQueryFilter.Swappable
    GemAssetFilter.HasBalance -> AssetsQueryFilter.HasBalance
    GemAssetFilter.HasAvailableBalance -> AssetsQueryFilter.HasAvailableBalance
    is GemAssetFilter.Chains -> AssetsQueryFilter.Chains(chains.map { it.requireChain() })
    is GemAssetFilter.ChainsOrAssetIds -> AssetsQueryFilter.ChainsOrAssets(chains.map { it.requireChain() }, assetIds)
}
