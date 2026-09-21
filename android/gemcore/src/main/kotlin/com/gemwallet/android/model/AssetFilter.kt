package com.gemwallet.android.model

import com.wallet.core.primitives.Chain

sealed interface AssetFilter {
    data object Enabled : AssetFilter
    data object Buyable : AssetFilter
    data object Sellable : AssetFilter
    data object Swappable : AssetFilter
    data object HasBalance : AssetFilter
    data object HasAvailableBalance : AssetFilter
    data class ChainsOrAssetIds(val chains: List<Chain>, val ids: List<String>) : AssetFilter
    data class Chains(val chains: List<Chain>) : AssetFilter
}

fun Collection<AssetFilter>.chains(): List<Chain> = filterIsInstance<AssetFilter.Chains>().flatMap { it.chains }

fun Collection<AssetFilter>.chainsOrAssetIds(): AssetFilter.ChainsOrAssetIds? = filterIsInstance<AssetFilter.ChainsOrAssetIds>()
    .takeIf { it.isNotEmpty() }
    ?.let { filters -> AssetFilter.ChainsOrAssetIds(filters.flatMap { it.chains }, filters.flatMap { it.ids }) }
