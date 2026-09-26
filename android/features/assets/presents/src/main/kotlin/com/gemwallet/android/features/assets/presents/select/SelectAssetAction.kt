package com.gemwallet.android.features.assets.presents.select

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

sealed interface SelectAssetAction {
    data object Cancel : SelectAssetAction
    data object AddAsset : SelectAssetAction
    data object ClearFilters : SelectAssetAction
    data object OpenRecentsSheet : SelectAssetAction
    data object ShowAllAssets : SelectAssetAction
    data class Select(val asset: Asset) : SelectAssetAction
    data class SelectRecent(val asset: Asset) : SelectAssetAction
    data class ChainFilter(val chain: Chain) : SelectAssetAction
    data class BalanceFilter(val onlyWithBalance: Boolean) : SelectAssetAction
}
