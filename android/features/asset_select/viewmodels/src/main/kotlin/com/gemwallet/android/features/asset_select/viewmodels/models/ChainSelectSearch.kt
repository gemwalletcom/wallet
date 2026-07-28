package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.application.asset_select.coordinators.GetChainAssets
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

class ChainSelectSearch(
    private val getChainAssets: GetChainAssets,
    private val chain: Chain,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> = getChainAssets(chain)
}
