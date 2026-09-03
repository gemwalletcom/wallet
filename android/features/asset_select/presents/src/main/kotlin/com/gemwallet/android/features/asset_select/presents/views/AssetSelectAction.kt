package com.gemwallet.android.features.asset_select.presents.views

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

sealed interface AssetSelectAction {
    data object Cancel : AssetSelectAction
    data object AddAsset : AssetSelectAction
    data object ClearFilters : AssetSelectAction
    data object OpenRecentsSheet : AssetSelectAction
    data object ShowAllAssets : AssetSelectAction
    data class Select(val asset: Asset) : AssetSelectAction
    data class SelectRecent(val asset: Asset) : AssetSelectAction
    data class ChainFilter(val chain: Chain) : AssetSelectAction
    data class BalanceFilter(val onlyWithBalance: Boolean) : AssetSelectAction
}
