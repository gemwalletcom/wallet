package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.assets.coordinators.SetAssetPinned
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import uniffi.gemstone.GemAssetAction
import com.gemwallet.android.domains.asset.eligible
import com.gemwallet.android.domains.asset.recentFilters
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@HiltViewModel
class BuySelectViewModel @Inject constructor(
    getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    getRecentAssets: GetRecentAssets,
    updateRecentAsset: UpdateRecentAsset,
    switchAssetVisibility: SwitchAssetVisibility,
    setAssetPinned: SetAssetPinned,
    searchTokensCase: SearchTokensCase,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    updateRecentAsset,
    switchAssetVisibility,
    setAssetPinned,
    searchTokensCase,
    BuySelectSearch(searchSelectAssets),
) {
    override fun assetFilters() = GemAssetAction.BUY.recentFilters()
}

class BuySelectSearch(
    searchSelectAssets: SearchSelectAssets,
) : BaseSelectSearch(searchSelectAssets) {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return super.items(filters).map { items -> filter(items) }
    }

    override fun filter(items: List<AssetInfo>): List<AssetInfo> = GemAssetAction.BUY.eligible(items)
}
