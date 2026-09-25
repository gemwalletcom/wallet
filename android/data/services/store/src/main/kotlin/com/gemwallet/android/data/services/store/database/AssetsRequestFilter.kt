package com.gemwallet.android.data.services.store.database

import com.wallet.core.primitives.Chain

sealed interface AssetsRequestFilter {
    data object Enabled : AssetsRequestFilter
    data object Buyable : AssetsRequestFilter
    data object Sellable : AssetsRequestFilter
    data object Swappable : AssetsRequestFilter
    data object HasBalance : AssetsRequestFilter
    data object HasAvailableBalance : AssetsRequestFilter
    data class Chains(val chains: List<Chain>) : AssetsRequestFilter
    data class ChainsOrAssets(val chains: List<Chain>, val assetIds: List<String>) : AssetsRequestFilter
}

internal fun Collection<AssetsRequestFilter>.chains(): List<Chain> = filterIsInstance<AssetsRequestFilter.Chains>().flatMap { it.chains }

internal fun Collection<AssetsRequestFilter>.chainsOrAssets(): AssetsRequestFilter.ChainsOrAssets? = filterIsInstance<AssetsRequestFilter.ChainsOrAssets>()
    .takeIf { it.isNotEmpty() }
    ?.let { filters -> AssetsRequestFilter.ChainsOrAssets(filters.flatMap { it.chains }, filters.flatMap { it.assetIds }) }
