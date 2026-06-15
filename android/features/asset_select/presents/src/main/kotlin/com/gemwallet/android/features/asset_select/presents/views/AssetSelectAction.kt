package com.gemwallet.android.features.asset_select.presents.views

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain

sealed interface AssetSelectAction {
    data object Cancel : AssetSelectAction
    data object AddAsset : AssetSelectAction
    data object ClearFilters : AssetSelectAction
    data object OpenRecentsSheet : AssetSelectAction
    data object ShowAllAssets : AssetSelectAction
    data class Select(val assetId: AssetId) : AssetSelectAction
    data class SelectRecent(val assetId: AssetId) : AssetSelectAction
    data class ChainFilter(val chain: Chain) : AssetSelectAction
    data class BalanceFilter(val onlyWithBalance: Boolean) : AssetSelectAction
    data class SelectTag(val tag: AssetTag?) : AssetSelectAction
}
