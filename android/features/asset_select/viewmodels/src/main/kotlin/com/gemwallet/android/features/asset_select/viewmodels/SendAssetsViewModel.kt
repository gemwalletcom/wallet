package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.assets.coordinators.SetAssetPinned
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
open class SendSelectViewModel @Inject constructor(
    getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    getSelectAssetsInfo: GetSelectAssetsInfo,
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
    SendSelectSearch(searchSelectAssets, getSelectAssetsInfo),
    remoteSearch = false,
) {
    override fun assetFilters() = setOf(AssetFilter.HasBalance)
}

@OptIn(ExperimentalCoroutinesApi::class)
class SendSelectSearch(
    private val searchSelectAssets: SearchSelectAssets,
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
) : BaseSelectSearch(searchSelectAssets) {
    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters
            .map { filters -> filters?.query.orEmpty() }
            .flatMapLatest { query ->
                val source = if (query.isEmpty()) {
                    getSelectAssetsInfo()
                } else {
                    searchSelectAssets(query)
                }

                source.map(::filter)
            }
    }

    override fun filter(items: List<AssetInfo>): List<AssetInfo> = items.filter { it.balance.totalAmount != 0.0 }
}
